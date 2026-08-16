import test from "node:test";
import assert from "node:assert/strict";
import { FetchTransport } from "../dist/transport.js";
import { createFetchEndpoint, json } from "./support.ts";
test("transport sends JSON through the Fetch contract and retries a retryable response", async () => {
  let attempt = 0; const server = createFetchEndpoint((request) => { attempt += 1; return attempt === 1 ? json(503, { title: "busy", status: 503 }) : json(200, { accepted: JSON.parse(request.body) }); });
  const transport = new FetchTransport({ baseUrl: server.baseUrl, fetch: server.fetch, maxRetries: 1 });
  const value = await transport.request({ method: "POST", path: "/v1/projects", body: { name: "orbit" }, idempotencyKey: "request-0001" });
  assert.deepEqual(value, { accepted: { name: "orbit" } }); assert.equal(server.requests.length, 2); assert.equal(server.requests[1].headers["idempotency-key"], "request-0001");
});
