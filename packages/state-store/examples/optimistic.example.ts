import { DEFAULT_STUDIO_STATE, beginOptimistic, createEvent, enqueueOptimistic, optimisticView } from "../src/index.js";

const pending = enqueueOptimistic(beginOptimistic(DEFAULT_STUDIO_STATE), "save:world:1", createEvent({ type: "workspace.updated", patch: { projectId: "project:1", worldId: "world:1" } }, "client:1", 1));
console.log(optimisticView(pending).workspace.worldId);
