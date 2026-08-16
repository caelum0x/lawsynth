import { StateStore } from "../src/index.js";

const store = new StateStore();
store.dispatch({ kind: "workspace.update", patch: { projectId: "project:1", worldId: "world:1", route: "/projects/1/worlds/1" } });
store.dispatch({ kind: "panel.update", id: "timeline", patch: { open: false } });
console.log(store.snapshot());
