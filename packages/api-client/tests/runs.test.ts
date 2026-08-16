import test from "node:test";
import assert from "node:assert/strict";
import { LawSynthClient } from "../dist/index.js";
import { createFetchEndpoint, json } from "./support.ts";
test("run API percent-encodes resource identifiers", async () => {
  const server = createFetchEndpoint(() => json(200, { id: "run id" }));
  await new LawSynthClient({ baseUrl: server.baseUrl, fetch: server.fetch }).runs.get("run id"); assert.equal(server.requests[0].url, "/v1/runs/run%20id");
});
