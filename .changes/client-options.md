---
"better-posthog": minor
---

**Breaking:** The `ClientConfig` type is renamed into `ClientOptions`.

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
