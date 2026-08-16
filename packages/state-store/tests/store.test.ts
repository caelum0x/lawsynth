import assert from "node:assert/strict";
import test from "node:test";
import { StateStore } from "../src/store.js";

test("store commits deterministic revisions and publishes committed snapshots", () => {
  let now = 10;
  const store = new StateStore({ clock: () => now++, eventId: (() => { let n = 0; return () => `e:${++n}`; })() });
  const observed: number[] = [];
  store.subscribe((snapshot) => observed.push(snapshot.revision));
  store.dispatch({ kind: "workspace.update", patch: { projectId: "project:1", route: "/projects/1" } });
  store.dispatch({ kind: "selection.set", ids: ["var:x"], primaryId: "var:x" }, 1);
  assert.equal(store.revision, 2);
  assert.deepEqual(observed, [1, 2]);
  assert.equal(store.state.selection.primaryId, "var:x");
  assert.throws(() => store.dispatch({ kind: "workspace.clear" }, 0), /Revision conflict/);
});
