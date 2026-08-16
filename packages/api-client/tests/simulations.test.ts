import test from "node:test";
import assert from "node:assert/strict";
import { LawSynthClient } from "../dist/index.js";
import { createFetchEndpoint, json } from "./support.ts";
test("simulation API retrieves and cancels a submitted simulation", async () => {
  const server = createFetchEndpoint(() => json(200, { id: "s1", status: "queued" }));
  const simulations = new LawSynthClient({ baseUrl: server.baseUrl, fetch: server.fetch }).simulations;
  assert.equal((await simulations.get("s1")).id, "s1");
  await simulations.cancel("s1");
  assert.equal(server.requests[0].url, "/v1/simulations/s1");
  assert.equal(server.requests[1].headers["idempotency-key"], "cancel-s1");
});
