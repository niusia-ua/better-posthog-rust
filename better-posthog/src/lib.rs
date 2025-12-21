//! An ergonomic Rust SDK for [PostHog](https://posthog.com/).
//!
//! This crate offloads network I/O to a background thread, ensuring the main application remains non-blocking and error-free.
//!
//! # Quick Start
//!
//! ```no_run
//! use better_posthog::{events, Event};
//!
//! // Initialize the client.
//! let _guard = better_posthog::init(better_posthog::ClientOptions {
//!   api_key: Some("phc_your_api_key".into()),
//!   ..Default::default()
//! });
//!
//! // Capture events.
//! events::capture(Event::new("page_view", "user_123"));
//!
//! // Or use the builder pattern for events.
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

pub use client::{ApiKey, BeforeSendFn, ClientOptions, Host};
use client::{CLIENT, Client};

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
/// use better_posthog::{init, ClientOptions};
///
/// let _guard = init(ClientOptions::new("phc_your_api_key"));
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
/// If no API key is provided in the configuration, the client will not be initialized.
/// Event capture calls will be no-ops in this case.
///
/// # Panics
///
/// Panics if called more than once.
///
/// # Examples
///
/// ```no_run
/// let _guard = better_posthog::init(better_posthog::ClientOptions {
///   api_key: Some("phc_your_api_key".into()),
///   host: better_posthog::Host::EU,
///   shutdown_timeout: std::time::Duration::from_secs(5),
/// });
/// ```
pub fn init(options: ClientOptions) -> ClientGuard {
  let shutdown_timeout = options.shutdown_timeout;

  if options.api_key.is_none() {
    log::warn!("PostHog client not initialized: no API key provided");
    return ClientGuard { shutdown_timeout };
  }

  assert!(
    CLIENT.set(Client::new(options)).is_ok(),
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
  #[allow(clippy::option_if_let_else)]
  if let Some(client) = CLIENT.get() {
    client.worker.flush(timeout)
  } else {
    log::warn!("PostHog client not initialized");
    false
  }
}
