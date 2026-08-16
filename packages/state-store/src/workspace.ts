import { InvariantError } from "./errors.js";

export interface WorkspaceState {
  readonly projectId: string | undefined;
  readonly worldId: string | undefined;
  readonly runId: string | undefined;
  readonly route: string;
}

export const EMPTY_WORKSPACE: WorkspaceState = Object.freeze({
  projectId: undefined,
  worldId: undefined,
  runId: undefined,
  route: "/",
});

export type WorkspacePatch = Partial<Pick<WorkspaceState, "projectId" | "worldId" | "runId" | "route">>;

export function updateWorkspace(current: WorkspaceState, patch: WorkspacePatch): WorkspaceState {
  const next: WorkspaceState = {
    projectId: patch.projectId === undefined ? current.projectId : identifier(patch.projectId, "projectId"),
    worldId: patch.worldId === undefined ? current.worldId : identifier(patch.worldId, "worldId"),
    runId: patch.runId === undefined ? current.runId : identifier(patch.runId, "runId"),
    route: patch.route === undefined ? current.route : route(patch.route),
  };
  if (next.worldId !== undefined && next.projectId === undefined) throw new InvariantError("A world requires an active project");
  if (next.runId !== undefined && next.projectId === undefined) throw new InvariantError("A run requires an active project");
  return sameWorkspace(current, next) ? current : Object.freeze(next);
}

export function clearWorkspace(current: WorkspaceState): WorkspaceState {
  return current === EMPTY_WORKSPACE ? current : EMPTY_WORKSPACE;
}

export function validateWorkspace(value: WorkspaceState): WorkspaceState {
  return updateWorkspace(EMPTY_WORKSPACE, value);
}

function identifier(value: string, name: string): string {
  if (!/^[A-Za-z0-9][A-Za-z0-9._:-]{0,255}$/u.test(value)) throw new InvariantError(`${name} is not a valid identifier`);
  return value;
}

function route(value: string): string {
  if (!value.startsWith("/") || /[\r\n\0]/u.test(value)) throw new InvariantError("route must be an absolute safe path");
  return value;
}

function sameWorkspace(left: WorkspaceState, right: WorkspaceState): boolean {
  return left.projectId === right.projectId && left.worldId === right.worldId && left.runId === right.runId && left.route === right.route;
}
