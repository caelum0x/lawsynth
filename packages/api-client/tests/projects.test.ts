import test from "node:test";
import assert from "node:assert/strict";
import { LawSynthClient } from "../dist/index.js";
import { createFetchEndpoint, json } from "./support.ts";
test("project API encodes collection paths and idempotency keys", async () => {
  const server = createFetchEndpoint((request) => json(201, { id: "p1", name: JSON.parse(request.body).name }));
  const result = await new LawSynthClient({ baseUrl: server.baseUrl, fetch: server.fetch }).projects.create({ name: "gravity" }, "project-0001"); assert.equal(result.id, "p1"); assert.equal(server.requests[0].url, "/v1/projects");
});
