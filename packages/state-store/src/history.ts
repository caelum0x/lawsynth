import { InvariantError } from "./errors.js";

export interface HistoryEntry<E> { readonly revision: number; readonly event: E; }
export interface HistoryState<E> { readonly past: readonly HistoryEntry<E>[]; readonly future: readonly HistoryEntry<E>[]; readonly limit: number; }

export function createHistory<E>(limit = 100): HistoryState<E> {
  if (!Number.isSafeInteger(limit) || limit < 1 || limit > 10_000) throw new InvariantError("History limit must be an integer in 1..=10000");
  return Object.freeze({ past: Object.freeze([]), future: Object.freeze([]), limit });
}

export function recordHistory<E>(history: HistoryState<E>, entry: HistoryEntry<E>): HistoryState<E> {
  if (!Number.isSafeInteger(entry.revision) || entry.revision < 1) throw new InvariantError("History revision must be positive");
  const past = [...history.past, Object.freeze(entry)].slice(-history.limit);
  return Object.freeze({ ...history, past: Object.freeze(past), future: Object.freeze([]) });
}

export function moveUndo<E>(history: HistoryState<E>): HistoryState<E> {
  const entry = history.past.at(-1);
  if (!entry) return history;
  return Object.freeze({ ...history, past: Object.freeze(history.past.slice(0, -1)), future: Object.freeze([entry, ...history.future]) });
}

export function moveRedo<E>(history: HistoryState<E>): HistoryState<E> {
  const entry = history.future[0];
  if (!entry) return history;
  return Object.freeze({ ...history, past: Object.freeze([...history.past, entry]), future: Object.freeze(history.future.slice(1)) });
}
