// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  let options = better_posthog::ClientOptions {
    api_key: option_env!("POSTHOG_API_KEY").map(Into::into),
    host: better_posthog::Host::EU,
    before_send: vec![
      {
        // Initialize a scoped `Send`-compatible RNG.
        let mut rng = fastrand::Rng::new();

        // Return a `before_send` hook.
        Box::new(move |event| {
          let sample_rate = match event.event.as_str() {
            "button_click" => Some(0.5), // Process only a half of `button_click` events.
            _ => None,                   // Process all other events.
          };
          if let Some(sample_rate) = sample_rate
            && rng.f64() < sample_rate
          {
            Some(event)
          } else {
            None
          }
        })
      },
      Box::new(|mut event| {
        #[cfg(debug_assertions)]
        event.insert_property("environment", "development");
        #[cfg(not(debug_assertions))]
        event.insert_property("environment", "production");
        Some(event)
      }),
    ],
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
