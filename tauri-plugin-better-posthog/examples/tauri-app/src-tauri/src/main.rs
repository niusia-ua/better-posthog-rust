// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  let _guard =
    better_posthog::init(better_posthog::ClientConfig::new(env!("POSTHOG_API_KEY")).host(better_posthog::Host::EU));

  tauri::Builder::default()
    .plugin(tauri_plugin_better_posthog::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
