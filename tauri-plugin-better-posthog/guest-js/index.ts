import { invoke } from "@tauri-apps/api/core";

/**
 * Captures a single event.
 * @param event - The event name.
 * @param properties - Optional properties to be sent with the event.
 */
export async function capture_event(
  event: string,
  properties?: Record<string, any>,
) {
  await invoke("plugin:better-posthog|capture", { event, properties });
}

/**
 * Captures a batch of events.
 * @param events - An array of events to be captured.
 */
export async function batch_events(
  events: { event: string; properties?: Record<string, any> }[],
) {
  await invoke("plugin:better-posthog|batch", { events });
}
