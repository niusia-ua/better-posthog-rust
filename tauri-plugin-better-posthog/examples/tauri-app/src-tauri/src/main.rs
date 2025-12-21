// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  let options = better_posthog::ClientOptions {
    api_key: option_env!("POSTHOG_API_KEY").map(Into::into),
    host: better_posthog::Host::EU,
    ..Default::default()
  };
  let _guard = better_posthog::init(options);

  tauri::Builder::default()
    .plugin(
      tauri_plugin_log::Builder::new()
        .level(log::LevelFilter::Info)
        .level_for("better_posthog", log::LevelFilter::Trace)
        .level_for("tauri_plugin_better_posthog", log::LevelFilter::Trace)
        .build(),
    )
    .plugin(tauri_plugin_better_posthog::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
