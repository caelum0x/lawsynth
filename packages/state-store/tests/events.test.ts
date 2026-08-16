import assert from "node:assert/strict";
import test from "node:test";
import { compareEvents, createEvent } from "../src/events.js";

test("events order by timestamp then event identifier", () => {
  const a = createEvent({ type: "workspace.cleared" }, "b", 5);
  const b = createEvent({ type: "workspace.cleared" }, "a", 5);
  assert.ok(compareEvents(a, b) > 0);
});
