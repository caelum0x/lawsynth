import test from "node:test";
import assert from "node:assert/strict";
import { apiErrorFromResponse, retryAfterMilliseconds } from "../dist/errors.js";
test("HTTP error envelopes retain server code and correlation id", () => {
  const error = apiErrorFromResponse(409, { error: { code: "idempotency_conflict", message: "key reused", request_id: "req-1" } }, new Headers());
  assert.equal(error.code, "idempotency_conflict"); assert.equal(error.requestId, "req-1"); assert.equal(error.isRetryable, false);
  assert.equal(retryAfterMilliseconds("2"), 2000);
});
