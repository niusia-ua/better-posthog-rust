// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  let config = better_posthog::ClientConfig {
    api_key: option_env!("POSTHOG_API_KEY").map(ToString::to_string),
    host: better_posthog::Host::EU,
    ..Default::default()
  };
  let _guard = better_posthog::init(config);

  tauri::Builder::default()
    .plugin(tauri_plugin_better_posthog::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
