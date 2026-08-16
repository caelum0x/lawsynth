import { createWorldViewModel, type TrajectorySource, type ViewerWorld, type WorldViewModel } from "./viewer.js";
export interface ViewerWorkerRequest { readonly id: string; readonly type: "build"; readonly world: ViewerWorld; readonly trajectory?: TrajectorySource; }
export interface ViewerWorkerResponse { readonly id: string; readonly ok: true; readonly model: WorldViewModel; } | { readonly id: string; readonly ok: false; readonly error: string; };
/** Pure worker handler: a Worker host may call this from onmessage without DOM coupling. */
export function handleViewerWorkerRequest(request: ViewerWorkerRequest): ViewerWorkerResponse { try { return { id: request.id, ok: true, model: createWorldViewModel(request.world, request.trajectory) }; } catch (error) { return { id: request.id, ok: false, error: error instanceof Error ? error.message : String(error) }; } }
