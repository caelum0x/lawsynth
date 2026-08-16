import { assert, test } from "./assertions.js";
import { createPopover } from "../src/popover.js";
import { componentNode } from "../src/tokens.js";

test("closed popovers are hidden and linked to their trigger", () => {
  const popover = createPopover({ id: "help", triggerId: "help-button", open: false, content: [componentNode("p", {}, { text: "Details" })] });
  assert.equal(popover.attributes.hidden, true);
  assert.equal(popover.attributes["aria-labelledby"], "help-button");
});
