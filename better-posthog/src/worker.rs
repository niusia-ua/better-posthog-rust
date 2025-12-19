use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use crate::Event;
use crate::client::ClientConfig;
use crate::context::saturate_event;

/// Messages that can be sent to the worker thread.
enum Task {
  /// A single event to capture.
  Capture(Event),
  /// A batch of events to send together.
  Batch(Vec<Event>),
  /// Flush request with acknowledgment channel.
  Flush(SyncSender<()>),
  /// Shutdown signal.
  Shutdown,
}

/// Payload for single event capture (`/i/v0/e/`).
#[derive(Serialize)]
struct CapturePayload<'a> {
  api_key: &'a str,
  event: &'a str,
  distinct_id: &'a str,
  properties: &'a HashMap<String, Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  timestamp: Option<&'a str>,
}

/// Payload for batch event capture (`/batch/`).
#[derive(Serialize)]
struct BatchPayload<'a> {
  api_key: &'a str,
  batch: Vec<BatchEvent>,
}

/// Single event within a batch. Note: `distinct_id` goes inside `properties`.
#[derive(Serialize)]
struct BatchEvent {
  event: String,
  properties: HashMap<String, Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  timestamp: Option<String>,
}

/// Background worker thread for sending events to PostHog.
pub struct Worker {
  sender: SyncSender<Task>,
  shutdown: Arc<AtomicBool>,
  handle: Option<JoinHandle<()>>,
}

impl Worker {
  /// Creates a new worker with a background thread for sending events.
  pub fn new(config: ClientConfig) -> Self {
    let (sender, receiver) = sync_channel(256);
    let shutdown = Arc::new(AtomicBool::new(false));

    let handle = thread::Builder::new()
      .name("better-posthog-worker".into())
      .spawn({
        let shutdown = shutdown.clone();

        let http_client = reqwest::blocking::Client::new();
        move || {
          for task in receiver {
            if shutdown.load(Ordering::SeqCst) {
              return;
            }

            match task {
              Task::Capture(mut event) => {
                saturate_event(&mut event);
                send_capture(&http_client, &config, &event);
              }
              Task::Batch(mut events) => {
                for event in &mut events {
                  saturate_event(event);
                }
                send_batch(&http_client, &config, &events);
              }
              Task::Flush(sender) => {
                sender.send(()).ok();
              }
              Task::Shutdown => {
                return;
              }
            }
          }
        }
      })
      .ok();

    Self {
      sender,
      shutdown,
      handle,
    }
  }

  /// Sends a single event to PostHog.
  ///
  /// If the queue is full, the event is dropped and a warning is logged.
  pub fn capture(&self, event: Event) {
    if let Err(e) = self.sender.try_send(Task::Capture(event)) {
      log::warn!("PostHog event dropped: {e}");
    }
  }

  /// Sends a batch of events to PostHog.
  ///
  /// If the queue is full, the batch is dropped and a warning is logged.
  pub fn batch(&self, events: Vec<Event>) {
    if let Err(e) = self.sender.try_send(Task::Batch(events)) {
      log::warn!("PostHog batch dropped: {e}");
    }
  }

  /// Flushes pending events, waiting up to the specified timeout.
  ///
  /// Returns `true` if the flush completed within the timeout.
  pub fn flush(&self, timeout: Duration) -> bool {
    let (sender, receiver) = sync_channel(1);
    let _ = self.sender.send(Task::Flush(sender));
    receiver.recv_timeout(timeout).is_ok()
  }
}

impl Drop for Worker {
  fn drop(&mut self) {
    self.shutdown.store(true, Ordering::SeqCst);
    let _ = self.sender.send(Task::Shutdown);
    if let Some(handle) = self.handle.take() {
      handle.join().ok();
    }
  }
}

/// Sends a single event to PostHog via `/i/v0/e/`.
fn send_capture(client: &reqwest::blocking::Client, config: &ClientConfig, event: &Event) {
  let url = config.host.capture_url();
  let payload = CapturePayload {
    api_key: config.api_key.as_ref().expect("API key must be present"),
    event: &event.event,
    distinct_id: &event.distinct_id,
    properties: &event.properties,
    timestamp: event.timestamp.as_deref(),
  };

  match serde_json::to_string(&payload) {
    Ok(body) => {
      let result = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send();

      match result {
        Ok(response) if response.status().is_success() => {}
        Ok(response) if response.status().as_u16() == 401 => {
          log::error!("PostHog authentication failed: invalid API key");
        }
        Ok(response) => {
          log::error!("PostHog request failed with status: {}", response.status());
        }
        Err(e) => {
          log::error!("Failed to send event to PostHog: {e}");
        }
      }
    }
    Err(e) => {
      log::error!("Failed to serialize event: {e}");
    }
  }
}

/// Sends a batch of events to PostHog via `/batch/`.
fn send_batch(client: &reqwest::blocking::Client, config: &ClientConfig, events: &[Event]) {
  let url = config.host.batch_url();
  let payload = BatchPayload {
    api_key: config.api_key.as_ref().expect("API key must be present"),
    batch: events
      .iter()
      .map(|event| {
        let mut properties = event.properties.clone();
        properties.insert("distinct_id".to_string(), Value::String(event.distinct_id.clone()));
        BatchEvent {
          event: event.event.clone(),
          timestamp: event.timestamp.clone(),
          properties,
        }
      })
      .collect(),
  };

  match serde_json::to_string(&payload) {
    Ok(body) => {
      let result = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send();

      match result {
        Ok(response) if response.status().is_success() => {}
        Ok(response) if response.status().as_u16() == 401 => {
          log::error!("PostHog authentication failed: invalid API key");
        }
        Ok(response) => {
          log::error!("PostHog batch request failed with status: {}", response.status());
        }
        Err(e) => {
          log::error!("Failed to send batch to PostHog: {e}");
        }
      }
    }
    Err(e) => {
      log::error!("Failed to serialize batch: {e}");
    }
  }
}
