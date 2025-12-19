//! Plugin state management for session and identity.

/// Plugin state containing identity and session information.
pub struct PluginState {
  /// Resolved distinct ID.
  /// `None` means anonymous mode (transient UUID v7 per event).
  distinct_id: Option<String>,

  /// Session ID for the current application lifecycle.
  /// Generated as UUID v4 at the plugin initialization.
  session_id: String,
}

impl PluginState {
  /// Creates a new plugin state with the given distinct ID.
  ///
  /// Generates a new session ID (UUID v4) that persists for the application lifecycle.
  pub fn new(distinct_id: Option<String>) -> Self {
    Self {
      distinct_id,
      session_id: uuid::Uuid::new_v4().to_string(),
    }
  }

  /// Returns the session ID for the current application lifecycle.
  pub fn session_id(&self) -> &str {
    &self.session_id
  }

  /// Returns the distinct ID if available.
  ///
  /// Returns `None` for anonymous mode.
  pub fn distinct_id(&self) -> Option<&str> {
    self.distinct_id.as_deref()
  }
}
