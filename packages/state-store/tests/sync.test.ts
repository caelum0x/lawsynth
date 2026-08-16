import assert from "node:assert/strict";
import test from "node:test";
import { createEvent } from "../src/events.js";
import { eventsAfter, mergeEventLogs } from "../src/sync.js";

test("event log merging deduplicates exactly and preserves deterministic order", () => {
  const first = createEvent({ type: "workspace.cleared" }, "a", 1);
  const second = createEvent({ type: "workspace.cleared" }, "b", 1);
  const merged = mergeEventLogs([second], [first, second]);
  assert.deepEqual(merged.map((event) => event.eventId), ["a", "b"]);
  assert.deepEqual(eventsAfter(merged, "a").map((event) => event.eventId), ["b"]);
});
