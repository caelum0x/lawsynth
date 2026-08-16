import type { PlaygroundTheme } from "./theme.js";

export interface PlaygroundEmbedOptions {
  readonly example?: string;
  readonly shared?: string;
  readonly theme?: PlaygroundTheme;
  readonly readOnly?: boolean;
}

const SAFE_EXAMPLE = /^[a-z0-9][a-z0-9-]{1,63}$/u;

export function playgroundEmbedUrl(base: string | URL, options: PlaygroundEmbedOptions = {}): string {
  const url = new URL(base);
  url.searchParams.set("embed", "1");
  if (options.example !== undefined) {
    if (!SAFE_EXAMPLE.test(options.example)) throw new RangeError("example id is invalid");
    url.searchParams.set("example", options.example);
  }
  if (options.shared !== undefined) {
    if (!/^[A-Za-z0-9_-]{1,100000}$/u.test(options.shared)) throw new RangeError("shared payload is invalid");
    url.hash = `world=${options.shared}`;
  }
  if (options.theme !== undefined) url.searchParams.set("theme", options.theme);
  if (options.readOnly === true) url.searchParams.set("readonly", "1");
  return url.toString();
}

export function parseEmbedOptions(input: string | URL): PlaygroundEmbedOptions {
  const url = input instanceof URL ? input : new URL(input, "https://playground.invalid");
  const example = url.searchParams.get("example") ?? undefined;
  const themeValue = url.searchParams.get("theme");
  const shared = new URLSearchParams(url.hash.slice(1)).get("world") ?? undefined;
  if (example !== undefined && !SAFE_EXAMPLE.test(example)) throw new RangeError("example id is invalid");
  return Object.freeze({
    ...(example === undefined ? {} : { example }),
    ...(shared === undefined ? {} : { shared }),
    ...(themeValue === "paper" || themeValue === "midnight" ? { theme: themeValue } : {}),
    readOnly: url.searchParams.get("readonly") === "1",
  });
}
