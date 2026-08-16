import { componentNode, type ComponentNode } from "./tokens.js";

export interface TabSpec { readonly id: string; readonly label: string; readonly panel: readonly ComponentNode[]; readonly disabled?: boolean; }
export interface TabsProps { readonly id: string; readonly tabs: readonly TabSpec[]; readonly selectedId: string; readonly selectAction?: string; }

export function createTabs(props: TabsProps): ComponentNode {
  if (!props.id.trim() || props.tabs.length === 0) throw new RangeError("tabs require an id and at least one tab");
  const ids = new Set<string>();
  for (const tab of props.tabs) {
    if (!tab.id.trim() || !tab.label.trim() || ids.has(tab.id)) throw new RangeError("tabs require unique ids and labels");
    ids.add(tab.id);
  }
  const selected = props.tabs.find((tab) => tab.id === props.selectedId);
  if (selected === undefined || selected.disabled) throw new RangeError("selected tab must exist and be enabled");
  const tabId = (id: string) => `${props.id}-tab-${id}`;
  const panelId = (id: string) => `${props.id}-panel-${id}`;
  return componentNode("div", {}, { children: [
    componentNode("div", {}, { role: "tablist", children: props.tabs.map((tab) => componentNode("button", {
      id: tabId(tab.id), type: "button", role: "tab", "aria-controls": panelId(tab.id), "aria-selected": tab.id === props.selectedId,
      ...(tab.disabled ? { disabled: true } : {}), tabindex: tab.id === props.selectedId ? 0 : -1,
    }, { text: tab.label, events: props.selectAction === undefined || tab.disabled ? [] : [{ type: "select", action: `${props.selectAction}:${tab.id}` }] })) }),
    ...props.tabs.map((tab) => componentNode("section", { id: panelId(tab.id), role: "tabpanel", "aria-labelledby": tabId(tab.id), ...(tab.id === props.selectedId ? {} : { hidden: true }) }, { children: tab.panel })),
  ] });
}
