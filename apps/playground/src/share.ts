import type { WorldDefinition } from "@lawsynth/world-schema";
import { PlaygroundError } from "./errors.js";

export interface SharedPlayground {
  readonly version: 1;
  readonly world: WorldDefinition;
  readonly parameters?: Readonly<Record<string, number>>;
}

function encodeBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/u, "");
}

function decodeBase64Url(value: string): Uint8Array {
  if (!/^[A-Za-z0-9_-]+$/u.test(value)) throw new TypeError("share payload contains invalid characters");
  const normalized = value.replaceAll("-", "+").replaceAll("_", "/");
  const binary = atob(normalized + "=".repeat((4 - normalized.length % 4) % 4));
  return Uint8Array.from(binary, (character) => character.charCodeAt(0));
}

function validateParameters(parameters: SharedPlayground["parameters"]): void {
  for (const [name, value] of Object.entries(parameters ?? {})) {
    if (!name.trim() || !Number.isFinite(value)) throw new TypeError("shared parameter overrides must be named and finite");
  }
}

export function createShareUrl(base: string | URL, payload: SharedPlayground, maximumBytes = 64 * 1024): string {
  validateParameters(payload.parameters);
  const bytes = new TextEncoder().encode(JSON.stringify(payload));
  if (bytes.byteLength > maximumBytes) {
    throw new PlaygroundError("limit-exceeded", "share payload is too large; export a file instead");
  }
  const url = new URL(base);
  url.hash = `world=${encodeBase64Url(bytes)}`;
  return url.toString();
}

export function parseShareUrl(input: string | URL, maximumBytes = 64 * 1024): SharedPlayground | undefined {
  const url = input instanceof URL ? input : new URL(input, "https://playground.invalid");
  const encoded = new URLSearchParams(url.hash.slice(1)).get("world");
  if (encoded === null) return undefined;
  try {
    const bytes = decodeBase64Url(encoded);
    if (bytes.byteLength > maximumBytes) throw new PlaygroundError("limit-exceeded", "shared world is too large");
    const value = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)) as SharedPlayground;
    if (value.version !== 1 || typeof value.world !== "object" || value.world === null) throw new TypeError("invalid share payload");
    validateParameters(value.parameters);
    return value;
  } catch (error) {
    if (error instanceof PlaygroundError) throw error;
    throw new PlaygroundError("share-failed", "share link is malformed", error);
  }
}
