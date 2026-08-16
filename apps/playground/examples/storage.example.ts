import type { WorldDefinition } from "@lawsynth/world-schema";
import { PlaygroundStorage, type KeyValueStorage, type SavedPlayground } from "../src/storage.js";

/** Persist a validated local world through a browser-compatible key-value store (for example, localStorage). */
export function saveLocalPlayground(
  storage: KeyValueStorage,
  input: Omit<SavedPlayground, "version" | "world"> & { readonly world: WorldDefinition },
): PlaygroundStorage {
  const playgrounds = new PlaygroundStorage(storage);
  playgrounds.save({ ...input, version: 1, world: input.world });
  return playgrounds;
}

/** Load only a record whose id was explicitly requested; corrupt records remain typed storage errors. */
export function loadLocalPlayground(storage: KeyValueStorage, id: string): SavedPlayground | undefined {
  return new PlaygroundStorage(storage).load(id);
}
