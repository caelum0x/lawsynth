import { assert, test } from "./assertions.js";
import { createTooltip } from "../src/tooltip.js";

test("tooltip exposes role and hidden state", () => {
  const tooltip = createTooltip({ id: "tip", triggerId: "run", text: "Run discovery", visible: false });
  assert.equal(tooltip.role, "tooltip");
  assert.equal(tooltip.attributes.hidden, true);
});
