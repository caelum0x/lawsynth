import test from "node:test";
import assert from "node:assert/strict";
import { LawSynthClient } from "../dist/index.js";
import { createFetchEndpoint, json } from "./support.ts";
test("world API includes an explicit revision when requested", async () => {
  const server = createFetchEndpoint(() => json(200, { id: "w1" }));
  await new LawSynthClient({ baseUrl: server.baseUrl, fetch: server.fetch }).worlds.get("w1", 2); assert.equal(server.requests[0].url, "/v1/worlds/w1?revision=2");
});
