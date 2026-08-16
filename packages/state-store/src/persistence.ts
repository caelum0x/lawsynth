import { PersistenceError } from "./errors.js";
import { validatePanels } from "./panels.js";
import { validatePreferences } from "./preferences.js";
import { validateSelection } from "./selection.js";
import type { StudioState } from "./store.js";
import { validateWorkspace } from "./workspace.js";

export const PERSISTENCE_VERSION = 1;
export interface PersistedState { readonly version: typeof PERSISTENCE_VERSION; readonly state: StudioState; }
export interface PersistenceAdapter { load(key: string): Promise<string | undefined>; save(key: string, value: string): Promise<void>; remove(key: string): Promise<void>; }

export function serializeState(state: StudioState): string { return JSON.stringify({ version: PERSISTENCE_VERSION, state }); }
export function deserializeState(serialized: string): StudioState {
  let value: unknown;
  try { value = JSON.parse(serialized); } catch (cause) { throw new PersistenceError("Persisted state is not valid JSON", { cause }); }
  if (!isRecord(value) || value.version !== PERSISTENCE_VERSION || !isRecord(value.state)) throw new PersistenceError("Persisted state has an unsupported shape or version");
  try {
    const state = value.state;
    return Object.freeze({ workspace: validateWorkspace(asWorkspace(state.workspace)), selection: validateSelection(asSelection(state.selection)), panels: validatePanels(asPanels(state.panels)), preferences: validatePreferences(asPreferences(state.preferences)) });
  } catch (cause) { throw new PersistenceError("Persisted state violates store invariants", { cause }); }
}
export async function loadState(adapter: PersistenceAdapter, key: string): Promise<StudioState | undefined> { const raw = await adapter.load(key); return raw === undefined ? undefined : deserializeState(raw); }
export async function saveState(adapter: PersistenceAdapter, key: string, state: StudioState): Promise<void> { await adapter.save(key, serializeState(state)); }

function isRecord(value: unknown): value is Record<string, unknown> { return typeof value === "object" && value !== null && !Array.isArray(value); }
function asWorkspace(value: unknown): StudioState["workspace"] { if (!isRecord(value)) throw new TypeError("workspace must be object"); return value as unknown as StudioState["workspace"]; }
function asSelection(value: unknown): StudioState["selection"] { if (!isRecord(value) || !Array.isArray(value.ids)) throw new TypeError("selection must be object"); return value as unknown as StudioState["selection"]; }
function asPanels(value: unknown): StudioState["panels"] { if (!isRecord(value)) throw new TypeError("panels must be object"); return value as unknown as StudioState["panels"]; }
function asPreferences(value: unknown): StudioState["preferences"] { if (!isRecord(value)) throw new TypeError("preferences must be object"); return value as unknown as StudioState["preferences"]; }
