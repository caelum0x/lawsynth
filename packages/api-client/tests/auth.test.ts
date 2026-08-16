import test from "node:test";
import assert from "node:assert/strict";
import { ApiKeyAuth, BearerTokenAuth } from "../dist/auth.js";
test("authentication providers produce validated HTTP headers", async () => {
  assert.deepEqual(await new BearerTokenAuth(() => "token-123").headers(), { Authorization: "Bearer token-123" });
  assert.deepEqual(new ApiKeyAuth("secret", "X-Service-Key").headers(), { "X-Service-Key": "secret" });
  await assert.rejects(() => new BearerTokenAuth("bad\nvalue").headers(), TypeError);
});
