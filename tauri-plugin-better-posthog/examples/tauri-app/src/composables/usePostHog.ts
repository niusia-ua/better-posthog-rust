import posthog from "posthog-js/dist/module.no-external";
import { captureEvent } from "tauri-plugin-better-posthog";

export function usePostHog() {
  posthog.init("dummy_api_key", {
    // Disable web navigation capturing (the plugin tracks sessions manually).
    capture_pageview: false,
    capture_pageleave: false,

    // Disable auto capturing.
    autocapture: false,
    rageclick: false,
    capture_dead_clicks: false,
    capture_exceptions: false,
    capture_heatmaps: false,
    capture_performance: false,

    // Disable marketing features.
    save_referrer: false,
    save_campaign_params: false,

    // Disable loading of remote configs.
    disable_external_dependency_loading: true,

    // Disable pertistance (the data that should be persistent is stored on the Tauri's side).
    disable_persistence: false,

    // Disable extra features.
    disable_surveys: true,
    disable_session_recording: true,
    disable_scroll_properties: true,
    disable_web_experiments: true,
    enable_recording_console_log: false,
    enable_heatmaps: false,

    // Disable `/flags`-dependent features.
    advanced_enable_surveys: false,
    advanced_disable_flags: true,
    advanced_disable_feature_flags: true,
    advanced_disable_toolbar_metrics: true,
    advanced_disable_feature_flags_on_first_load: true,
    advanced_only_evaluate_survey_feature_flags: true,

    // Pass events through the Rust backend.
    before_send: [
      (captureResult) => {
        if (captureResult) {
          const { event, properties } = captureResult;
          // eslint-disable-next-line no-console
          captureEvent(event, properties).catch(console.log);
        }
        return null;
      },
    ],
  });

  return { posthog };
}
