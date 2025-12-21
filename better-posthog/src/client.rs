use std::fmt;
use std::sync::OnceLock;
use std::time::Duration;

use crate::Event;
use crate::worker::Worker;

/// Hook that can modify or discard events before sending.
///
/// Each hook receives an owned [`Event`] and can:
/// - Return `Some(event)` to pass the (possibly modified) event to the next hook
/// - Return `None` to discard the event and stop further processing
///
/// Hooks are executed in order after the event is enriched with library/OS context.
/// If any hook panics, the event is discarded and an error is logged.
///
/// # Thread Safety
///
/// Hooks run in a background worker thread, so they must be `Send + 'static`.
/// If you need randomness (e.g., for sampling), use a `Send`-compatible RNG like [`fastrand::Rng`](https://docs.rs/fastrand) initialized before the closure.
///
/// # Example
///
/// ```
/// let options = better_posthog::ClientOptions {
///   api_key: Some("phc_your_api_key".into()),
///   before_send: vec![{
///     // Initialize a scoped `Send`-compatible RNG.
///     let mut rng = fastrand::Rng::new();
///
///     // Return a `before_send` hook.
///     Box::new(move |event| {
///       let sample_rate = match event.event.as_str() {
///         "button_click" => 0.5, // Process only a half of `button_click` events.
///         _ => 1.0, // Process all other events.
///       };
///       if rng.f64() < sample_rate { Some(event) } else { None }
///     })
///   }],
///   ..Default::default()
/// };
/// ```
pub type BeforeSendFn = Box<dyn FnMut(Event) -> Option<Event> + Send + 'static>;

/// Global client instance.
pub static CLIENT: OnceLock<Client> = OnceLock::new();

/// Internal client state holding the worker.
pub struct Client {
  pub worker: Worker,
}

impl Client {
  /// Creates a new client from the given configuration.
  pub fn new(options: ClientOptions) -> Self {
    let worker = Worker::new(options);
    Self { worker }
  }
}

/// Configuration for the PostHog client.
pub struct ClientOptions {
  /// The PostHog API key. If `None`, the client will not be initialized.
  pub api_key: Option<ApiKey>,
  /// The target PostHog host.
  pub host: Host,
  /// Timeout for graceful shutdown (default: 2 seconds).
  pub shutdown_timeout: Duration,
  /// Hooks to modify or filter events before sending.
  pub before_send: Vec<BeforeSendFn>,
}

impl fmt::Debug for ClientOptions {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ClientOptions")
      .field("api_key", &self.api_key)
      .field("host", &self.host)
      .field("shutdown_timeout", &self.shutdown_timeout)
      .field("before_send", &format!("[{} hooks]", self.before_send.len()))
      .finish()
  }
}

impl Default for ClientOptions {
  fn default() -> Self {
    Self {
      api_key: None,
      host: Host::default(),
      shutdown_timeout: Duration::from_secs(2),
      before_send: Vec::new(),
    }
  }
}

impl ClientOptions {
  /// Creates a new `ClientOptions` with the given API key and default settings.
  pub fn new<T: Into<ApiKey>>(api_key: T) -> Self {
    Self {
      api_key: Some(api_key.into()),
      ..Default::default()
    }
  }
}

impl<T: Into<ApiKey>> From<T> for ClientOptions {
  fn from(api_key: T) -> Self {
    Self::new(api_key)
  }
}

impl<T: Into<ApiKey>> From<(T, Self)> for ClientOptions {
  fn from((api_key, mut options): (T, Self)) -> Self {
    options.api_key = Some(api_key.into());
    options
  }
}

/// PostHog API key newtype.
#[derive(Debug, Clone)]
pub struct ApiKey(String);

impl ApiKey {
  /// Returns the API key as a string slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl From<&str> for ApiKey {
  fn from(key: &str) -> Self {
    Self(key.to_owned())
  }
}

impl From<String> for ApiKey {
  fn from(key: String) -> Self {
    Self(key)
  }
}

/// Target PostHog environment for event submission.
#[derive(Debug, Clone, Default)]
pub enum Host {
  /// US PostHog cloud instance (<https://us.i.posthog.com>).
  #[default]
  US,
  /// EU PostHog cloud instance (<https://eu.i.posthog.com>).
  EU,
  /// Custom self-hosted PostHog instance.
  Custom(String),
}

impl Host {
  /// Returns the base URL for this host.
  #[must_use]
  pub const fn base_url(&self) -> &str {
    match self {
      Self::US => "https://us.i.posthog.com",
      Self::EU => "https://eu.i.posthog.com",
      Self::Custom(url) => url.as_str(),
    }
  }

  /// Returns the single event capture endpoint URL.
  #[must_use]
  pub fn capture_url(&self) -> String {
    format!("{}/i/v0/e/", self.base_url())
  }

  /// Returns the batch event endpoint URL.
  #[must_use]
  pub fn batch_url(&self) -> String {
    format!("{}/batch/", self.base_url())
  }
}
