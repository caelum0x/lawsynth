import { InvariantError } from "./errors.js";
import type { StateEvent } from "./events.js";
import { reduceState } from "./mutations.js";
import type { StudioState } from "./store.js";

export interface OptimisticEntry { readonly token: string; readonly event: StateEvent; }
export interface OptimisticState { readonly base: StudioState; readonly pending: readonly OptimisticEntry[]; }

export function beginOptimistic(base: StudioState): OptimisticState { return Object.freeze({ base, pending: Object.freeze([]) }); }
export function optimisticView(state: OptimisticState): StudioState { return state.pending.reduce((current, entry) => reduceState(current, entry.event), state.base); }
export function enqueueOptimistic(state: OptimisticState, token: string, event: StateEvent): OptimisticState {
  if (!/^[A-Za-z0-9][A-Za-z0-9._:-]{0,255}$/u.test(token)) throw new InvariantError("Optimistic token is invalid");
  if (state.pending.some((entry) => entry.token === token)) throw new InvariantError(`Optimistic token already exists: ${token}`);
  return Object.freeze({ ...state, pending: Object.freeze([...state.pending, Object.freeze({ token, event })]) });
}
/** Acknowledge replaces the base with the authoritative state and removes the matching local overlay. */
export function acknowledgeOptimistic(state: OptimisticState, token: string, base: StudioState): OptimisticState { return Object.freeze({ base, pending: Object.freeze(state.pending.filter((entry) => entry.token !== token)) }); }
export function rejectOptimistic(state: OptimisticState, token: string): OptimisticState { return Object.freeze({ ...state, pending: Object.freeze(state.pending.filter((entry) => entry.token !== token)) }); }
