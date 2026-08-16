import test from "node:test";
import assert from "node:assert/strict";
import { LawSynthClient } from "../dist/index.js";
import { createFetchEndpoint, json } from "./support.ts";
test("high-level client composes resource clients over its transport", async () => {
  const server = createFetchEndpoint(() => json(200, { status: "ok" }));
  assert.deepEqual(await new LawSynthClient({ baseUrl: server.baseUrl, fetch: server.fetch }).health(), { status: "ok" });
});
