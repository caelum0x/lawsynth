import { assert, test } from "./assertions.js";
import { createDialog } from "../src/dialog.js";

test("open modal dialog requires a dismiss route and focus contract", () => {
  const dialog = createDialog({ id: "confirm", title: "Confirm", content: [], open: true, dismissAction: "confirm.close", initialFocusId: "cancel", returnFocusId: "trigger" });
  assert.equal(dialog.role, "dialog");
  assert.equal(dialog.focus?.trap, true);
  assert.deepEqual(dialog.events, [{ type: "dismiss", action: "confirm.close" }]);
  assert.throws(() => createDialog({ id: "confirm", title: "Confirm", content: [], open: true }), /dismissal/);
});
