import { canonicalJson, sha256Hex } from "../src/hash.js";

const payload = { laws: [{ target: "x", expression: { kind: "constant", value: 0 } }], id: "hashable" };

export const canonicalPayload = canonicalJson(payload);
export async function payloadSha256(): Promise<string> { return sha256Hex(payload); }
