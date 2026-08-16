import type { CandidateSummary } from "@lawsynth/api-client";
import type { Law, WorldDefinition } from "@lawsynth/world-schema";
import { equationView, type EquationView } from "@lawsynth/world-viewer";

export interface CandidateComparison {
  readonly candidate: CandidateSummary;
  readonly rank: number;
  readonly selected: boolean;
  readonly scoreDelta: number;
}

export function compareCandidates(candidates: readonly CandidateSummary[], selectedId?: string): readonly CandidateComparison[] {
  const seen = new Set<string>();
  const sorted = [...candidates].map((candidate) => {
    if (!candidate.id.trim() || seen.has(candidate.id)) throw new RangeError(`candidate ids must be unique: ${candidate.id}`);
    if (!Number.isFinite(candidate.score)) throw new RangeError(`candidate ${candidate.id} score must be finite`);
    seen.add(candidate.id); return candidate;
  }).sort((left, right) => right.score - left.score || left.id.localeCompare(right.id));
  const best = sorted[0]?.score ?? 0;
  return Object.freeze(sorted.map((candidate, index) => Object.freeze({ candidate, rank: index + 1, selected: candidate.id === selectedId, scoreDelta: best - candidate.score })));
}

export interface EquationCatalog {
  readonly equations: readonly EquationView[];
  readonly byTarget: ReadonlyMap<string, readonly EquationView[]>;
  readonly disabled: number;
}

export function equationCatalog(world: WorldDefinition): EquationCatalog {
  const equations = world.laws.map((law) => equationView(law, world.time.symbol ?? "t"));
  const grouped = new Map<string, EquationView[]>();
  for (const equation of equations) {
    const target = equation.target ?? equation.id;
    grouped.set(target, [...(grouped.get(target) ?? []), equation]);
  }
  return Object.freeze({ equations: Object.freeze(equations), byTarget: grouped, disabled: equations.filter((equation) => !equation.enabled).length });
}

export function replaceLaw(world: WorldDefinition, law: Law): WorldDefinition {
  const index = world.laws.findIndex((candidate) => candidate.id === law.id);
  if (index < 0) throw new RangeError(`unknown law: ${law.id}`);
  const laws = [...world.laws]; laws[index] = law;
  return { ...world, laws };
}
