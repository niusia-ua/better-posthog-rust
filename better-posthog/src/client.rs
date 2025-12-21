use std::sync::OnceLock;
use std::time::Duration;

use crate::worker::Worker;

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
#[derive(Debug, Clone)]
pub struct ClientOptions {
  /// The PostHog API key. If `None`, the client will not be initialized.
  pub api_key: Option<ApiKey>,
  /// The target PostHog host.
  pub host: Host,
  /// Timeout for graceful shutdown (default: 2 seconds).
  pub shutdown_timeout: Duration,
}

impl Default for ClientOptions {
  fn default() -> Self {
    Self {
      api_key: None,
      host: Host::default(),
      shutdown_timeout: Duration::from_secs(2),
    }
  }
}

impl<T: Into<ApiKey>> From<T> for ClientOptions {
  fn from(api_key: T) -> Self {
    Self {
      api_key: Some(api_key.into()),
      ..Default::default()
    }
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
