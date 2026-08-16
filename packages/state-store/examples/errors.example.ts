import { EMPTY_WORKSPACE, RevisionConflictError, StateStore, updateWorkspace } from "../src/index.js";

try { updateWorkspace(EMPTY_WORKSPACE, { worldId: "world:orphan" }); } catch (error) { console.error((error as Error).message); }
const store = new StateStore();
try { store.dispatch({ kind: "workspace.clear" }, 1); } catch (error) { console.error(error instanceof RevisionConflictError); }
