import { InvariantError } from "./errors.js";

export interface SelectionState {
  readonly ids: readonly string[];
  readonly primaryId: string | undefined;
  readonly hoveredId: string | undefined;
}

export const EMPTY_SELECTION: SelectionState = Object.freeze({ ids: Object.freeze([]), primaryId: undefined, hoveredId: undefined });

export function select(ids: Iterable<string>, primaryId?: string): SelectionState {
  const unique = [...new Set(ids)];
  for (const id of unique) validateId(id);
  if (primaryId !== undefined) {
    validateId(primaryId);
    if (!unique.includes(primaryId)) throw new InvariantError("primaryId must be included in selection");
  }
  return unique.length === 0 ? EMPTY_SELECTION : Object.freeze({ ids: Object.freeze(unique), primaryId: primaryId ?? unique[0], hoveredId: undefined });
}

export function toggleSelection(current: SelectionState, id: string): SelectionState {
  validateId(id);
  return current.ids.includes(id) ? select(current.ids.filter((candidate) => candidate !== id), current.primaryId === id ? undefined : current.primaryId) : select([...current.ids, id], current.primaryId ?? id);
}

export function setHovered(current: SelectionState, id: string | undefined): SelectionState {
  if (id !== undefined) validateId(id);
  return current.hoveredId === id ? current : Object.freeze({ ...current, hoveredId: id });
}

export function clearSelection(current: SelectionState): SelectionState {
  return current === EMPTY_SELECTION ? current : EMPTY_SELECTION;
}

export function validateSelection(value: SelectionState): SelectionState {
  const next = select(value.ids, value.primaryId);
  return setHovered(next, value.hoveredId);
}

function validateId(value: string): void {
  if (!/^[A-Za-z0-9][A-Za-z0-9._:-]{0,255}$/u.test(value)) throw new InvariantError("Selection identifiers must be safe non-empty identifiers");
}
