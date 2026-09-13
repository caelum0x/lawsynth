import assert from "node:assert/strict";
import test from "node:test";

import worker, { canonicalRedirectUrl } from "../worker.mjs";

test("www requests redirect to the apex and preserve path and query", async () => {
  const request = new Request("https://www.lawsynth.dev/docs/methods/sparse/stlsq?solver=stlsq");
  const response = await worker.fetch(request, {
    ASSETS: { fetch: () => assert.fail("redirects must not fetch a static asset") },
  });

  assert.equal(response.status, 301);
  assert.equal(
    response.headers.get("location"),
    "https://lawsynth.dev/docs/methods/sparse/stlsq?solver=stlsq",
  );
});

test("apex and preview hosts continue to the static asset binding", async () => {
  for (const url of ["https://lawsynth.dev/docs/guide", "https://lawsynth.example.workers.dev/"]) {
    let fetchedRequest;
    const request = new Request(url);
    const response = await worker.fetch(request, {
      ASSETS: {
        fetch(value) {
          fetchedRequest = value;
          return new Response(null, { status: 204 });
        },
      },
    });

    assert.equal(response.status, 204);
    assert.equal(fetchedRequest, request);
    assert.equal(canonicalRedirectUrl(url), null);
  }
});
