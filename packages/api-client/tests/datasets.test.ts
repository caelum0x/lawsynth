import test from "node:test";
import assert from "node:assert/strict";
import { LawSynthClient } from "../dist/index.js";
import { createFetchEndpoint, json } from "./support.ts";
test("dataset list sends project scope and page cursor", async () => {
  const server = createFetchEndpoint(() => json(200, { items: [], next: null }));
  await new LawSynthClient({ baseUrl: server.baseUrl, fetch: server.fetch }).datasets.list("p1", { after: "c1", limit: 2 }); assert.match(server.requests[0].url, /project_id=p1/); assert.match(server.requests[0].url, /cursor=c1/);
});
