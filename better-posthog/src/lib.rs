//! An ergonomic Rust SDK for [PostHog](https://posthog.com/).
//!
//! This crate offloads network I/O to a background thread, ensuring the main application remains non-blocking and error-free.
//!
//! # Quick Start
//!
//! ```no_run
//! use better_posthog::{init, events, Event, ClientConfig};
//!
//! // Initialize the client.
//! let _guard = init(ClientConfig::new("phc_your_api_key"));
//!
//! // Capture events.
//! events::capture(Event::new("page_view", "user_123"));
//!
//! // Or use the builder pattern.
//! events::capture(
//!   Event::builder()
//!     .event("button_click")
//!     .distinct_id("user_123")
//!     .property("button_id", "submit")
//!     .build()
//! );
//!
//! // Batch multiple events.
//! events::batch(vec![
//!   Event::new("event_1", "user_123"),
//!   Event::new("event_2", "user_123"),
//! ]);
//!
//! // Guard is dropped here, triggering graceful shutdown.
//! ```

mod client;
mod context;
mod worker;

use client::{CLIENT, Client};
pub use client::{ClientConfig, Host};

pub mod events;
pub use events::{Event, EventBuilder};

/// Guard that manages the PostHog client lifecycle.
///
/// When dropped, this guard triggers graceful shutdown of the background worker,
/// attempting to flush pending events within the configured timeout.
///
/// # Examples
///
/// ```no_run
/// use better_posthog::{init, ClientConfig};
///
/// let _guard = init(ClientConfig::new("phc_your_api_key"));
///
/// // ... application code ...
///
/// // Guard is dropped here, triggering graceful shutdown
/// ```
#[must_use = "ClientGuard must be held for the duration of the application"]
pub struct ClientGuard {
  shutdown_timeout: std::time::Duration,
}

impl Drop for ClientGuard {
  fn drop(&mut self) {
    if let Some(client) = CLIENT.get()
      && !client.worker.flush(self.shutdown_timeout)
    {
      log::warn!(
        "PostHog shutdown timed out after {:?}, some events may be lost",
        self.shutdown_timeout
      );
    }
  }
}

/// Initializes the PostHog client with the given configuration.
///
/// Returns a [`ClientGuard`] that must be held for the duration of the application.
/// When the guard is dropped, it triggers graceful shutdown of the background worker.
///
/// # Panics
///
/// Panics if called more than once.
///
/// # Examples
///
/// ```no_run
/// use better_posthog::{init, ClientConfig, Host};
///
/// let config = ClientConfig::new("phc_your_api_key")
///   .host(Host::EU)
///   .shutdown_timeout(std::time::Duration::from_secs(5));
///
/// let _guard = init(config);
/// ```
pub fn init(config: ClientConfig) -> ClientGuard {
  let shutdown_timeout = config.shutdown_timeout;

  assert!(
    CLIENT.set(Client::new(config)).is_ok(),
    "PostHog client already initialized"
  );

  ClientGuard { shutdown_timeout }
}

/// Flushes pending events, waiting up to the specified timeout.
///
/// Returns `true` if the flush completed within the timeout.
///
/// # Examples
///
/// ```no_run
/// if !better_posthog::flush(std::time::Duration::from_secs(5)) {
///   eprintln!("Flush timed out");
/// }
/// ```
pub fn flush(timeout: std::time::Duration) -> bool {
  CLIENT.get().map_or_else(
    || {
      log::warn!("PostHog client not initialized");
      false
    },
    |client| client.worker.flush(timeout),
  )
}
