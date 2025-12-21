# `better_posthog`

An ergonomic Rust SDK for [PostHog](https://posthog.com/).

## Features

- Configurable API client.
- Non-blocking and error-free event capture with background worker thread.
- Builder pattern for flexible event construction.
- Automatic OS and library metadata enrichment.
- Support for events editing, filtering, and sampling via the `before_send` option.
- Graceful shutdown with configurable timeout.

## Usage

```rust
use better_posthog::{events, Event};

fn main() {
  // Initialize the client.
  let _guard = better_posthog::init(better_posthog::ClientOptions {
    api_key: Some("phc_your_api_key".into()),
    ..Default::default()
  });

  // Capture a single event.
  events::capture(Event::new("page_view", "user_123"));

  // Use the builder for more control.
  events::capture(
    Event::builder()
      .event("button_click")
      .distinct_id("user_123")
      .property("button_id", "submit")
      .build()
  );

  // Batch multiple events.
  events::batch(vec![
    Event::new("event_1", "user_123"),
    Event::new("event_2", "user_123"),
  ]);

  // Guard drop triggers graceful shutdown.
}
```

## Configuration

```rust
let _guard = better_posthog::init(better_posthog::ClientOptions {
  api_key: Some("phc_your_api_key".into()),
  host: better_posthog::Host::EU, // or `Host::US`, `Host::Custom(String::from("https://..."))`
  before_send: vec![], // Hooks to edit, filter, or sample events before sending.
  shutdown_timeout: std::time::Duration::from_secs(5),
});
```

## License

[MIT](../License)
