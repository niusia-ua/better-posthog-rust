---
"better-posthog": minor
---

A new `before_send` option allows you to modify, filter, or sample events before they are sent:

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
