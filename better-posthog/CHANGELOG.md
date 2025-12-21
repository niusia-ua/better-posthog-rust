# Changelog

## \[0.2.0]

- [`bd4e625`](https://github.com/niusia-ua/better-posthog-rust/commit/bd4e625edc78477242db03cb954f4a51648298fa) ([#10](https://github.com/niusia-ua/better-posthog-rust/pull/10)) A new `before_send` option allows you to modify, filter, or sample events before they are sent:

  ```rs
  let _guard = better_posthog::init(better_posthog::ClientOptions {
    api_key: Some("phc_your_api_key".into()),
    before_send: vec![
      // Events sampling.
      {
        // Initialize a scoped `Send`-compatible RNG.
        let mut rng = fastrand::Rng::new();

        // Return a `before_send` hook.
        Box::new(move |event| {
          let sample_rate = match event.event.as_str() {
            "button_click" => Some(0.5), // Process only a half of `button_click` events.
            _ => None,                   // Process all other events.
          };
          if let Some(sample_rate) = sample_rate && rng.f64() < sample_rate {
            Some(event)
          } else {
            None
          }
        })
      },

      // Events saturating.
      Box::new(|mut event| {
        #[cfg(debug_assertions)]
        event.insert_property("environment", "development");
        #[cfg(not(debug_assertions))]
        event.insert_property("environment", "production");
        Some(event)
      }),
    ],
    ..Default::default()
  });
  ```

  Hooks run in the background worker thread after events are enriched with library/OS context.
  Panics in user-provided hooks are caught and logged; the event is discarded safely.

- [`e0164d3`](https://github.com/niusia-ua/better-posthog-rust/commit/e0164d394da8a6ab6dcbc1972c565ddf5157f9e5) ([#8](https://github.com/niusia-ua/better-posthog-rust/pull/8)) **Breaking:** The `ClientConfig` type is renamed into `ClientOptions`.

  A new `ApiKey` type is introduced.
  In addition, you can now instantiate the PostHog client in more convenient ways:

  ```rs
  // 1. Verbose.
  let _guard = better_posthog::init(better_posthog::ClientOptions {
    api_key: Some("phc_your_api_key".into()),
    ..Default::default()
  });

  // 2. Compact.
  let _guard = better_posthog::init(("phc_your_api_key", better_posthog::ClientOptions::default()));

  // 3. API key-only with default options.
  let _guard = better_posthog::init("phc_your_api_key");
  ```

- [`a83a17c`](https://github.com/niusia-ua/better-posthog-rust/commit/a83a17c3af24b8c010eef6693215a38b0f8e1051) ([#9](https://github.com/niusia-ua/better-posthog-rust/pull/9)) Added trace logs for debugging.

## \[0.1.0]

- [`640f14c`](https://github.com/niusia-ua/better-posthog-rust/commit/640f14c557758e4dec4ebbeef7eb4e1fd926b1e1) ([#7](https://github.com/niusia-ua/better-posthog-rust/pull/7)) Initial release.
