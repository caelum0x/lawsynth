import { DEFAULT_SETTINGS, mergeSettings, parseSettings } from "../src/settings.js";

/** Restore persisted settings, then apply an explicit operator-selected endpoint. */
export function restoreSettings(serialized: string | undefined, apiBaseUrl: string) {
  const restored = serialized === undefined ? DEFAULT_SETTINGS : parseSettings(serialized);
  return mergeSettings(restored, { apiBaseUrl });
}
