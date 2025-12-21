use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use crate::Event;
use crate::client::ClientOptions;
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
  pub fn new(mut options: ClientOptions) -> Self {
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
                log::trace!("Processing capture task for event: {}", event.event);
                saturate_event(&mut event);
                if let Some(event) = apply_before_send(&mut options, event) {
                  send_capture(&http_client, &options, &event);
                } else {
                  log::trace!("Event was dropped by before_send hook");
                }
              }
              Task::Batch(events) => {
                let events_count = events.len();
                log::trace!("Processing batch task with {events_count} events");

                let events: Vec<Event> = events
                  .into_iter()
                  .filter_map(|mut event| {
                    saturate_event(&mut event);
                    apply_before_send(&mut options, event)
                  })
                  .collect();
                if events_count != events.len() {
                  log::trace!(
                    "{} events were dropped by before_send hook",
                    events_count - events.len()
                  );
                }

                if !events.is_empty() {
                  send_batch(&http_client, &options, &events);
                }
              }
              Task::Flush(sender) => {
                log::trace!("Processing flush task");
                sender.send(()).ok();
              }
              Task::Shutdown => {
                log::trace!("Shutting down worker thread");
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
    log::trace!("Capturing {} event", event.event);
    if let Err(e) = self.sender.try_send(Task::Capture(event)) {
      log::warn!("PostHog event dropped: {e}");
    }
  }

  /// Sends a batch of events to PostHog.
  ///
  /// If the queue is full, the batch is dropped and a warning is logged.
  pub fn batch(&self, events: Vec<Event>) {
    log::trace!("Capturing batch with {} events", events.len());
    if let Err(e) = self.sender.try_send(Task::Batch(events)) {
      log::warn!("PostHog batch dropped: {e}");
    }
  }

  /// Flushes pending events, waiting up to the specified timeout.
  ///
  /// Returns `true` if the flush completed within the timeout.
  pub fn flush(&self, timeout: Duration) -> bool {
    log::trace!("Flushing event with {timeout:?} timeout");
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

/// Applies all `before_send` hooks to an event.
///
/// Returns `Some(event)` if the event should be sent, `None` if it was discarded.
/// The event is discarded on panic.
fn apply_before_send(options: &mut ClientOptions, mut event: Event) -> Option<Event> {
  for hook in &mut options.before_send {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| hook(event))) {
      Ok(Some(e)) => event = e,
      Ok(None) => return None,
      Err(_) => {
        log::error!("Panic in before_send hook, discarding event");
        return None;
      }
    }
  }
  Some(event)
}

/// Sends a single event to PostHog via `/i/v0/e/`.
fn send_capture(client: &reqwest::blocking::Client, options: &ClientOptions, event: &Event) {
  let url = options.host.capture_url();
  let payload = CapturePayload {
    api_key: options.api_key.as_ref().expect("API key must be present").as_str(),
    event: &event.event,
    distinct_id: &event.distinct_id,
    properties: &event.properties,
    timestamp: event.timestamp.as_deref(),
  };

  match serde_json::to_string(&payload) {
    Ok(body) => {
      log::trace!("Serialized payload size: {} bytes", body.len());
      let result = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send();

      match result {
        Ok(response) if response.status().is_success() => {
          log::trace!("Capture request successful: status {}", response.status());
        }
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
fn send_batch(client: &reqwest::blocking::Client, options: &ClientOptions, events: &[Event]) {
  let url = options.host.batch_url();
  let payload = BatchPayload {
    api_key: options.api_key.as_ref().expect("API key must be present").as_str(),
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
      log::trace!("Serialized batch payload size: {} bytes", body.len());
      let result = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send();

      match result {
        Ok(response) if response.status().is_success() => {
          log::trace!("Batch request successful: status {}", response.status());
        }
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
