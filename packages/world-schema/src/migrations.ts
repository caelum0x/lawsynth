import { parseSemanticVersion } from "./types.js";
import { CURRENT_WORLD_VERSION, type WorldDefinition } from "./world.js";

export type MigrationDocument = Record<string, unknown> & { formatVersion: string };
export type Migration = (document: Readonly<MigrationDocument>) => MigrationDocument;

export interface MigrationStep {
  from: string;
  to: string;
  migrate: Migration;
}

const steps = new Map<string, MigrationStep>();

export function registerMigration(step: MigrationStep): void {
  if (!parseSemanticVersion(step.from) || !parseSemanticVersion(step.to) || step.from === step.to) {
    throw new Error(`Invalid migration ${step.from} -> ${step.to}`);
  }
  if (steps.has(step.from)) throw new Error(`Migration from ${step.from} is already registered`);
  steps.set(step.from, step);
}

export function migrationPath(from: string, to = CURRENT_WORLD_VERSION): readonly MigrationStep[] {
  if (from === to) return [];
  const path: MigrationStep[] = [];
  const visited = new Set<string>();
  let current = from;
  while (current !== to) {
    if (visited.has(current)) throw new Error(`Migration cycle detected at ${current}`);
    visited.add(current);
    const step = steps.get(current);
    if (!step) throw new Error(`No migration registered from ${current} to ${to}`);
    path.push(step);
    current = step.to;
  }
  return path;
}

export function migrateDocument(document: Readonly<MigrationDocument>, to = CURRENT_WORLD_VERSION): MigrationDocument {
  let current: MigrationDocument = structuredCloneSafe(document);
  for (const step of migrationPath(current.formatVersion, to)) {
    const next = step.migrate(current);
    if (next.formatVersion !== step.to) {
      throw new Error(`Migration ${step.from} -> ${step.to} returned version ${next.formatVersion}`);
    }
    current = next;
  }
  return current;
}

export function migrateWorld(document: unknown, to = CURRENT_WORLD_VERSION): WorldDefinition {
  if (!isMigrationDocument(document)) throw new TypeError("World document has no formatVersion");
  return migrateDocument(document, to) as unknown as WorldDefinition;
}

export function isMigrationDocument(value: unknown): value is MigrationDocument {
  return typeof value === "object" && value !== null && typeof (value as { formatVersion?: unknown }).formatVersion === "string";
}

function structuredCloneSafe<T>(value: T): T {
  if (typeof structuredClone === "function") return structuredClone(value);
  return JSON.parse(JSON.stringify(value)) as T;
}
