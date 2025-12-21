# `tauri-plugin-better-posthog`

Tauri integration with PostHog.

## Installation

```bash
npm install posthog-js tauri-plugin-better-posthog
cargo add better-posthog tauri-plugin-better-posthog
```

## Backend Setup

Initialize the PostHog client and register the plugin in your Tauri application:

```rust
fn main() {
  // Initialize the client (keep the guard alive for the application lifetime).
  let _guard = better_posthog::init(better_posthog::ClientOptions {
    api_key: Some("phc_your_api_key".to_string()),
    ..Default::default()
  });

  // Register the plugin.
  tauri::Builder::default()
    .plugin(tauri_plugin_better_posthog::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
```

## Frontend Setup

Configure `posthog-js` to route all events through the Rust backend:

```javascript
import posthog from "posthog-js";
import { captureEvent } from "tauri-plugin-better-posthog";

posthog.init("dummy_api_key", {
  // other options...

  // Route all events through the Rust backend.
  before_send: [
    // Keep this function last.
    (captureResult) => {
      if (captureResult) {
        const { event, properties } = captureResult;
        captureEvent(event, properties).catch(console.error);
      }
      // Return `null` to prevent `posthog-js` from sending directly.
      return null;
    },
  ],
});
```

## License

[MIT](../LICENSE)
