import { InvariantError } from "./errors.js";

export type Theme = "system" | "light" | "dark";
export type Density = "compact" | "comfortable";
export interface PreferencesState { readonly theme: Theme; readonly density: Density; readonly reducedMotion: boolean; readonly telemetryEnabled: boolean; }
export const DEFAULT_PREFERENCES: PreferencesState = Object.freeze({ theme: "system", density: "comfortable", reducedMotion: false, telemetryEnabled: false });

export function updatePreferences(current: PreferencesState, patch: Partial<PreferencesState>): PreferencesState {
  const next = { ...current, ...patch };
  if (next.theme !== "system" && next.theme !== "light" && next.theme !== "dark") throw new InvariantError("Unknown color theme");
  if (next.density !== "compact" && next.density !== "comfortable") throw new InvariantError("Unknown density");
  if (typeof next.reducedMotion !== "boolean" || typeof next.telemetryEnabled !== "boolean") throw new InvariantError("Preference booleans must be boolean");
  return next.theme === current.theme && next.density === current.density && next.reducedMotion === current.reducedMotion && next.telemetryEnabled === current.telemetryEnabled ? current : Object.freeze(next);
}

export function validatePreferences(value: PreferencesState): PreferencesState { return updatePreferences(DEFAULT_PREFERENCES, value); }
