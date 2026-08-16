import { InvariantError } from "./errors.js";
import type { StateEvent } from "./events.js";
import type { StudioState } from "./store.js";

/** Reversible command description. The exact inverse is supplied by domain code, avoiding guesswork. */
export interface UndoEntry { readonly event: StateEvent; readonly inverse: StateEvent; }
export interface UndoStack { readonly undo: readonly UndoEntry[]; readonly redo: readonly UndoEntry[]; readonly limit: number; }

export function createUndoStack(limit = 100): UndoStack { if (!Number.isSafeInteger(limit) || limit < 1) throw new InvariantError("Undo limit must be positive"); return Object.freeze({ undo: Object.freeze([]), redo: Object.freeze([]), limit }); }
export function pushUndo(stack: UndoStack, entry: UndoEntry): UndoStack { return Object.freeze({ ...stack, undo: Object.freeze([...stack.undo, Object.freeze(entry)].slice(-stack.limit)), redo: Object.freeze([]) }); }
export function takeUndo(stack: UndoStack): { readonly stack: UndoStack; readonly event?: StateEvent } { const entry = stack.undo.at(-1); return entry === undefined ? { stack } : { event: entry.inverse, stack: Object.freeze({ ...stack, undo: Object.freeze(stack.undo.slice(0, -1)), redo: Object.freeze([entry, ...stack.redo]) }) }; }
export function takeRedo(stack: UndoStack): { readonly stack: UndoStack; readonly event?: StateEvent } { const entry = stack.redo[0]; return entry === undefined ? { stack } : { event: entry.event, stack: Object.freeze({ ...stack, undo: Object.freeze([...stack.undo, entry]), redo: Object.freeze(stack.redo.slice(1)) }) }; }
/** Type-level anchor for application code that holds a state plus an undo stack. */
export interface UndoableState { readonly state: StudioState; readonly undo: UndoStack; }
