import { extractApiOperations, groupApiOperations } from "../src/api_reference.js";
import { deepEqual, equal, test, throws } from "./assertions.js";

test("OpenAPI operations retain documentation details in deterministic order", () => {
  const operations = extractApiOperations({
    paths: {
      "/v1/worlds": {
        get: { operationId: "listWorlds", summary: "List worlds", tags: ["Worlds"], parameters: [{ name: "limit", in: "query", required: false, schema: { type: "integer" } }], responses: { 200: {}, 401: {} } },
        post: { summary: "Create a world", tags: ["Worlds", "Writes"], responses: { 201: {} } },
      },
      "/v1/runs/{id}": { get: { tags: [], parameters: [{ name: "id", in: "path", required: true }], responses: { 200: {} } } },
    },
  });
  deepEqual(operations.map((operation) => [operation.method, operation.path, operation.id]), [["GET", "/v1/runs/{id}", "get-v1-runs-id"], ["GET", "/v1/worlds", "listWorlds"], ["POST", "/v1/worlds", "post-v1-worlds"]]);
  deepEqual(operations[1]!.parameters, [{ name: "limit", location: "query", required: false, schemaType: "integer" }]);
  deepEqual(operations[1]!.responseCodes, ["200", "401"]);
  const groups = groupApiOperations(operations);
  equal(groups.get("Worlds")!.length, 2);
  equal(groups.get("Other")![0]!.path, "/v1/runs/{id}");
});

test("OpenAPI extraction rejects documents without a path map", () => {
  throws(() => extractApiOperations({ openapi: "3.1.0" }), /paths/);
});
