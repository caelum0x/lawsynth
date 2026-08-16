import { assert, test } from "./assertions.js";
import { createTable } from "../src/table.js";

test("table rejects incomplete rows", () => {
  const table = createTable({ caption: "Scores", columns: [{ id: "law", label: "Law" }, { id: "score", label: "Score", numeric: true }], rows: [{ id: "candidate-1", cells: { law: "x' = ax", score: 0.8 } }] });
  assert.equal(table.tag, "table");
  assert.throws(() => createTable({ caption: "Scores", columns: [{ id: "law", label: "Law" }], rows: [{ id: "broken", cells: { unexpected: "x" } }] }), /does not match/);
});
