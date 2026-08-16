import { assert, test } from "./assertions.js";
import { createSelect } from "../src/select.js";

test("select validates its selected value", () => {
  const select = createSelect({ id: "solver", label: "Solver", value: "rk4", options: [{ value: "euler", label: "Euler" }, { value: "rk4", label: "RK4" }] });
  assert.equal(select.children[1]?.tag, "select");
  assert.throws(() => createSelect({ id: "solver", label: "Solver", value: "unknown", options: [{ value: "rk4", label: "RK4" }] }), /not an option/);
});
