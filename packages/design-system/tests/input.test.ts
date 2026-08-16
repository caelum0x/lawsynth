import { assert, test } from "./assertions.js";
import { createInput } from "../src/input.js";

test("input connects visible label, description, and error", () => {
  const input = createInput({ id: "name", label: "Name", description: "Shown in the bundle", error: "Name is required", required: true });
  const control = input.children[1];
  assert.equal(control?.attributes["aria-describedby"], "name-description name-error");
  assert.equal(control?.attributes["aria-invalid"], true);
  assert.equal(input.children[0]?.attributes.for, "name");
});
