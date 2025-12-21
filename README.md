# Better PostHog Rust SDK

An ergonomic Rust SDK for [PostHog](https://posthog.com/).

> [!IMPORTANT]
> **This is not an official PostHog Rust SDK.**
>
> This project was developed to solve the author's own problems and achieve their goals, though it can be helpful to other developers.
> If there are missed features or bugs, PRs and issues are always welcome.

## Crates

- [`better-posthog`](./better-posthog) - Core SDK.
- [`tauri-plugin-better-posthog`](./tauri-plugin-better-posthog) - Tauri integration.

## FAQ

<details>
  <summary>Why not <a href="https://github.com/PostHog/posthog-rs"><code>posthog-rs</code></a>?</summary>

  The official crate is not actively maintained, though it is currently served by a person from the PostHog team.
  Therefore, PRs that introduce even minor features aren't merged for month.

  So I decided to build my own crate for my own needs.
  I hope that one day the official crate will continue to develop, so that there will be no need for such variation of SDKs.
</details>

## License

[MIT](./LICENSE)
