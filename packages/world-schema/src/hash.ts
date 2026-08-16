const textEncoder = new TextEncoder();

/** Deterministic JSON with lexical object keys and JSON-compatible values. */
export function canonicalJson(value: unknown): string {
  const ancestors = new Set<object>();

  const encode = (input: unknown): string => {
    if (input === null) return "null";
    if (typeof input === "string" || typeof input === "boolean") return JSON.stringify(input);
    if (typeof input === "number") {
      if (!Number.isFinite(input)) throw new TypeError("Canonical JSON cannot encode a non-finite number");
      return Object.is(input, -0) ? "0" : JSON.stringify(input);
    }
    if (typeof input === "bigint" || typeof input === "function" || typeof input === "symbol" || input === undefined) {
      throw new TypeError(`Canonical JSON cannot encode ${typeof input}`);
    }
    if (typeof input !== "object") throw new TypeError("Unsupported canonical JSON value");
    if (ancestors.has(input)) throw new TypeError("Canonical JSON cannot encode cyclic data");

    ancestors.add(input);
    try {
      if (Array.isArray(input)) return `[${input.map(encode).join(",")}]`;
      const record = input as Record<string, unknown>;
      const entries = Object.keys(record)
        .filter((key) => record[key] !== undefined)
        .sort()
        .map((key) => `${JSON.stringify(key)}:${encode(record[key])}`);
      return `{${entries.join(",")}}`;
    } finally {
      ancestors.delete(input);
    }
  };

  return encode(value);
}

export async function sha256Hex(value: unknown): Promise<string> {
  if (!globalThis.crypto?.subtle) throw new Error("Web Crypto SHA-256 is unavailable in this runtime");
  const digest = await globalThis.crypto.subtle.digest("SHA-256", textEncoder.encode(canonicalJson(value)));
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

/** Fast deterministic cache key; never use this in place of bundle SHA-256. */
export function stableHash32(value: unknown): string {
  const bytes = textEncoder.encode(canonicalJson(value));
  let hash = 0x811c9dc5;
  for (const byte of bytes) {
    hash ^= byte;
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return hash.toString(16).padStart(8, "0");
}
