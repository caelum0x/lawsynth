import { StateStore, activeProjectId, isPanelOpen, selectedIds } from "../src/index.js";

const store = new StateStore();
store.dispatch({ kind: "workspace.update", patch: { projectId: "project:climate" } });
store.dispatch({ kind: "selection.set", ids: ["variable:temperature"] });
console.log({ project: activeProjectId(store.state), selection: selectedIds(store.state), inspectorOpen: isPanelOpen(store.state, "inspector") });
