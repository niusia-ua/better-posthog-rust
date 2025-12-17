//! Public API for capturing PostHog events.

use std::collections::HashMap;

use crate::client::CLIENT;

/// Captures a single event and sends it to PostHog.
///
/// If the client is not initialized or the queue is full, the event is dropped
/// and a warning is logged. This function never blocks.
///
/// # Examples
///
/// ```no_run
/// use better_posthog::{events, Event};
///
/// let event = Event::new("button_click", "user_123");
/// events::capture(event);
/// ```
pub fn capture(event: Event) {
  if let Some(client) = CLIENT.get() {
    client.worker.capture(event);
  }
}

/// Captures a batch of events and sends them to PostHog in a single request.
///
/// If the client is not initialized or the queue is full, the batch is dropped
/// and a warning is logged. This function never blocks.
///
/// # Examples
///
/// ```no_run
/// use better_posthog::{events, Event};
///
/// let events = vec![
///   Event::new("page_view", "user_123"),
///   Event::new("button_click", "user_123"),
/// ];
/// events::batch(events);
/// ```
pub fn batch(events: Vec<Event>) {
  if let Some(client) = CLIENT.get() {
    client.worker.batch(events);
  }
}

/// A PostHog analytics event.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Event {
  /// The event name.
  pub event: String,
  /// The user's unique identifier.
  pub distinct_id: String,
  /// Custom properties attached to the event.
  pub properties: HashMap<String, serde_json::Value>,
  /// Optional ISO 8601 timestamp. If not set, PostHog uses server time.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timestamp: Option<String>,
}

impl Event {
  /// Creates a new event with the given name and distinct ID.
  ///
  /// # Examples
  ///
  /// ```
  /// use better_posthog::Event;
  ///
  /// let event = Event::new("page_view", "user_123");
  /// ```
  pub fn new<S: Into<String>>(event: S, distinct_id: S) -> Self {
    Self {
      event: event.into(),
      distinct_id: distinct_id.into(),
      properties: HashMap::new(),
      timestamp: None,
    }
  }

  /// Creates a new event with a generated UUID v7 as the distinct ID.
  ///
  /// # Examples
  ///
  /// ```
  /// use better_posthog::Event;
  ///
  /// let event = Event::new_anonymous("anonymous_action");
  /// ```
  pub fn new_anonymous<S: Into<String>>(event: S) -> Self {
    Self {
      event: event.into(),
      distinct_id: uuid::Uuid::now_v7().to_string(),
      properties: HashMap::new(),
      timestamp: None,
    }
  }

  /// Returns a builder for constructing an event.
  ///
  /// # Examples
  ///
  /// ```
  /// use better_posthog::Event;
  ///
  /// let event = Event::builder()
  ///   .event("button_click")
  ///   .distinct_id("user_456")
  ///   .property("button_id", "submit")
  ///   .build();
  /// ```
  #[must_use]
  pub fn builder() -> EventBuilder {
    EventBuilder::default()
  }

  /// Inserts a property into the event.
  ///
  /// # Examples
  ///
  /// ```
  /// use better_posthog::Event;
  ///
  /// let mut event = Event::new("purchase", "user_789");
  /// event.insert_property("amount", 99.99);
  /// event.insert_property("currency", "USD");
  /// ```
  pub fn insert_property<K, V>(&mut self, key: K, value: V)
  where
    K: Into<String>,
    V: Into<serde_json::Value>,
  {
    self.properties.insert(key.into(), value.into());
  }
}

/// Builder for constructing [`Event`] instances.
#[derive(Debug, Default)]
pub struct EventBuilder {
  event: Option<String>,
  distinct_id: Option<String>,
  properties: HashMap<String, serde_json::Value>,
  timestamp: Option<String>,
}

impl EventBuilder {
  /// Sets the event name.
  #[must_use]
  pub fn event<S: Into<String>>(mut self, event: S) -> Self {
    self.event = Some(event.into());
    self
  }

  /// Sets the distinct ID.
  #[must_use]
  pub fn distinct_id<S: Into<String>>(mut self, distinct_id: S) -> Self {
    self.distinct_id = Some(distinct_id.into());
    self
  }

  /// Adds a property to the event.
  #[must_use]
  pub fn property<K: Into<String>, V: Into<serde_json::Value>>(mut self, key: K, value: V) -> Self {
    self.properties.insert(key.into(), value.into());
    self
  }

  /// Sets the timestamp (ISO 8601 format).
  #[must_use]
  pub fn timestamp<S: Into<String>>(mut self, timestamp: S) -> Self {
    self.timestamp = Some(timestamp.into());
    self
  }

  /// Builds the event.
  ///
  /// # Panics
  ///
  /// Panics if `event` is not set.
  #[must_use]
  pub fn build(self) -> Event {
    Event {
      event: self.event.expect("event name is required"),
      distinct_id: self.distinct_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string()),
      properties: self.properties,
      timestamp: self.timestamp,
    }
  }
}
