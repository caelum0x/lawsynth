export type StudioTheme = "system" | "light" | "dark";
export type StudioDensity = "comfortable" | "compact";

export interface StudioSettings {
  readonly apiBaseUrl: string;
  readonly theme: StudioTheme;
  readonly density: StudioDensity;
  readonly reducedMotion: boolean;
  readonly telemetryEnabled: boolean;
  readonly autosaveMs: number;
  readonly maxUploadBytes: number;
}

export const DEFAULT_SETTINGS: StudioSettings = Object.freeze({
  apiBaseUrl: "/api", theme: "system", density: "comfortable", reducedMotion: false,
  telemetryEnabled: false, autosaveMs: 1_500, maxUploadBytes: 512 * 1024 * 1024,
});

function validateUrl(value: string): string {
  if (!value.trim() || /[\r\n\0]/u.test(value)) throw new RangeError("API base URL is invalid");
  const parsed = new URL(value, "https://studio.invalid");
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") throw new RangeError("API base URL must use HTTP(S)");
  return value.replace(/\/$/u, "");
}

export function validateSettings(input: StudioSettings): StudioSettings {
  const apiBaseUrl = validateUrl(input.apiBaseUrl);
  if (!(["system", "light", "dark"] as const).includes(input.theme)) throw new RangeError("unknown Studio theme");
  if (input.density !== "comfortable" && input.density !== "compact") throw new RangeError("unknown Studio density");
  if (!Number.isSafeInteger(input.autosaveMs) || input.autosaveMs < 250 || input.autosaveMs > 60_000) throw new RangeError("autosaveMs must be in 250..60000");
  if (!Number.isSafeInteger(input.maxUploadBytes) || input.maxUploadBytes < 1024 || input.maxUploadBytes > 10 * 1024 ** 3) throw new RangeError("maxUploadBytes must be in 1 KiB..10 GiB");
  if (typeof input.reducedMotion !== "boolean" || typeof input.telemetryEnabled !== "boolean") throw new TypeError("settings flags must be boolean");
  return Object.freeze({ ...input, apiBaseUrl });
}

export function mergeSettings(base: StudioSettings, patch: Partial<StudioSettings>): StudioSettings {
  return validateSettings({ ...base, ...patch });
}

export function parseSettings(serialized: string): StudioSettings {
  let parsed: unknown;
  try { parsed = JSON.parse(serialized) as unknown; } catch (cause) { throw new SyntaxError("Studio settings are not valid JSON", { cause }); }
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) throw new TypeError("Studio settings must be an object");
  return mergeSettings(DEFAULT_SETTINGS, parsed as Partial<StudioSettings>);
}

export function effectiveTheme(settings: StudioSettings, prefersDark: boolean): Exclude<StudioTheme, "system"> {
  return settings.theme === "system" ? (prefersDark ? "dark" : "light") : settings.theme;
}
