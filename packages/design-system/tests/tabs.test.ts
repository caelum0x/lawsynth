import { assert, test } from "./assertions.js";
import { createTabs } from "../src/tabs.js";

test("tabs connect selected tab and panel through ARIA ids", () => {
  const tabs = createTabs({ id: "results", selectedId: "table", tabs: [{ id: "table", label: "Table", panel: [] }, { id: "plot", label: "Plot", panel: [] }], selectAction: "results.select" });
  const tabList = tabs.children[0];
  assert.equal(tabList?.role, "tablist");
  assert.equal(tabList?.children[0]?.attributes["aria-controls"], "results-panel-table");
  assert.equal(tabs.children[1]?.attributes.hidden, undefined);
  assert.equal(tabs.children[2]?.attributes.hidden, true);
});
