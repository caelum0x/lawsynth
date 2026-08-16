import { assert, test } from "./assertions.js";
import { createButton } from "../src/button.js";

test("button exposes native semantics and a serializable command", () => {
  const button = createButton({ id: "save", label: "Save", action: "world.save", busy: true });
  assert.equal(button.tag, "button");
  assert.equal(button.attributes["aria-busy"], true);
  assert.deepEqual(button.events, [{ type: "activate", action: "world.save" }]);
});

test("button rejects contradictory disabled busy state", () => {
  assert.throws(() => createButton({ id: "save", label: "Save", busy: true, disabled: true }), /busy/);
});
