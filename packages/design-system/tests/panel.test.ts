import { assert, test } from "./assertions.js";
import { createPanel } from "../src/panel.js";
import { componentNode } from "../src/tokens.js";

test("titled panel names its region", () => {
  const panel = createPanel({ id: "details", title: "Candidate details", children: [componentNode("p", {}, { text: "A sparse model" })] });
  assert.equal(panel.attributes["aria-labelledby"], "details-title");
  assert.equal(panel.children[0]?.text, "Candidate details");
});
