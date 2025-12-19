//! Tauri commands for frontend event capture.

use std::collections::HashMap;

use crate::PostHogExt as _;

/// Request payload for the `capture` command.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRequest {
  pub event: String,

  #[serde(default)]
  pub properties: Option<HashMap<String, serde_json::Value>>,
}

impl crate::PostHogEvent for CaptureRequest {
  fn name(&self) -> &str {
    &self.event
  }

  fn properties(&self) -> HashMap<String, serde_json::Value> {
    self.properties.clone().unwrap_or_default()
  }
}

/// Captures a single event from the frontend.
#[tauri::command]
pub async fn capture<R: tauri::Runtime>(
  event: String,
  properties: Option<HashMap<String, serde_json::Value>>,
  app_handle: tauri::AppHandle<R>,
) {
  app_handle.capture_event(CaptureRequest { event, properties });
}

/// Captures a batch of events from the frontend.
#[tauri::command]
pub async fn batch<R: tauri::Runtime>(events: Vec<CaptureRequest>, app_handle: tauri::AppHandle<R>) {
  app_handle.batch_events(&events);
}
