import test from "node:test";
import assert from "node:assert/strict";
import { pageQuery, paginate } from "../dist/pagination.js";
test("pagination maps opaque cursors and detects cycles", async () => {
  assert.deepEqual(pageQuery({ after: "opaque_1", limit: 2 }), { cursor: "opaque_1", limit: 2 });
  const pages = [{ items: [1, 2], next: "next" }, { items: [3], next: null }]; let index = 0;
  const values = []; for await (const item of paginate(async () => pages[index++])) values.push(item);
  assert.deepEqual(values, [1, 2, 3]);
});
