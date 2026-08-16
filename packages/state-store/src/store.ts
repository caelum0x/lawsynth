import { RevisionConflictError } from "./errors.js";
import type { Command } from "./commands.js";
import { commandEvent } from "./commands.js";
import type { StateEvent } from "./events.js";
import { createEvent } from "./events.js";
import { createHistory, moveRedo, moveUndo, recordHistory, type HistoryState } from "./history.js";
import { reduceState } from "./mutations.js";
import { DEFAULT_PANELS, type PanelsState } from "./panels.js";
import { DEFAULT_PREFERENCES, type PreferencesState } from "./preferences.js";
import { EMPTY_SELECTION, type SelectionState } from "./selection.js";
import { EMPTY_WORKSPACE, type WorkspaceState } from "./workspace.js";

export interface StudioState { readonly workspace: WorkspaceState; readonly selection: SelectionState; readonly panels: PanelsState; readonly preferences: PreferencesState; }
export const DEFAULT_STUDIO_STATE: StudioState = Object.freeze({ workspace: EMPTY_WORKSPACE, selection: EMPTY_SELECTION, panels: DEFAULT_PANELS, preferences: DEFAULT_PREFERENCES });

export interface StoreSnapshot { readonly revision: number; readonly state: StudioState; }
export type StoreListener = (snapshot: StoreSnapshot, event: StateEvent) => void;
export interface StateStoreOptions { readonly initial?: StudioState; readonly historyLimit?: number; readonly clock?: () => number; readonly eventId?: () => string; }

/** In-memory deterministic state store. Calls are synchronous and listeners observe committed snapshots only. */
export class StateStore {
  #state: StudioState;
  #revision = 0;
  #history: HistoryState<StateEvent>;
  #listeners = new Set<StoreListener>();
  #clock: () => number;
  #eventId: () => string;
  #sequence = 0;

  constructor(options: StateStoreOptions = {}) {
    this.#state = options.initial ?? DEFAULT_STUDIO_STATE;
    this.#history = createHistory(options.historyLimit);
    this.#clock = options.clock ?? (() => Date.now());
    this.#eventId = options.eventId ?? (() => `local:${++this.#sequence}`);
  }

  snapshot(): StoreSnapshot { return Object.freeze({ revision: this.#revision, state: this.#state }); }
  get state(): StudioState { return this.#state; }
  get revision(): number { return this.#revision; }
  get history(): HistoryState<StateEvent> { return this.#history; }

  subscribe(listener: StoreListener): () => void { this.#listeners.add(listener); return () => this.#listeners.delete(listener); }

  dispatch(command: Command, expectedRevision?: number): StoreSnapshot {
    return this.apply(createEvent(commandEvent(command), this.#eventId(), this.#clock()), expectedRevision);
  }

  apply(event: StateEvent, expectedRevision?: number): StoreSnapshot {
    if (expectedRevision !== undefined && expectedRevision !== this.#revision) throw new RevisionConflictError(expectedRevision, this.#revision);
    const next = reduceState(this.#state, event);
    if (next === this.#state || equivalentState(next, this.#state)) return this.snapshot();
    this.#state = freezeState(next);
    this.#revision += 1;
    this.#history = recordHistory(this.#history, { revision: this.#revision, event });
    const snapshot = this.snapshot();
    for (const listener of this.#listeners) listener(snapshot, event);
    return snapshot;
  }

  /** Records history movement only; event inversion belongs to the command domain. */
  markUndone(): void { this.#history = moveUndo(this.#history); }
  markRedone(): void { this.#history = moveRedo(this.#history); }
}

function freezeState(state: StudioState): StudioState { return Object.freeze(state); }
function equivalentState(left: StudioState, right: StudioState): boolean { return left.workspace === right.workspace && left.selection === right.selection && left.panels === right.panels && left.preferences === right.preferences; }
