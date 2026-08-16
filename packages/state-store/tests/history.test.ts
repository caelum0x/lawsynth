import assert from "node:assert/strict";
import test from "node:test";
import { createHistory, moveRedo, moveUndo, recordHistory } from "../src/history.js";

test("history bounds entries and moves undo/redo cursors", () => {
  let history = createHistory<string>(2);
  history = recordHistory(history, { revision: 1, event: "a" });
  history = recordHistory(history, { revision: 2, event: "b" });
  history = recordHistory(history, { revision: 3, event: "c" });
  assert.deepEqual(history.past.map((entry) => entry.event), ["b", "c"]);
  assert.equal(moveRedo(moveUndo(history)).past.at(-1)?.event, "c");
});
