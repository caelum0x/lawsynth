import type { RegimeDefinition, RegimeInterval, RegimeModel, WorldDefinition } from "@lawsynth/world-schema";
import { regimeTimelineForWorld, type RegimeTimeline } from "@lawsynth/world-viewer";

export interface RegimeWorkspace {
  readonly definitions: readonly RegimeDefinition[];
  readonly timeline?: RegimeTimeline;
  readonly uncoveredLawIds: readonly string[];
  readonly issues: readonly string[];
}

export function regimeWorkspace(world: WorldDefinition): RegimeWorkspace {
  const model = world.regimes;
  if (model === undefined) return Object.freeze({ definitions: Object.freeze([]), uncoveredLawIds: Object.freeze([]), issues: Object.freeze(["No regime model is declared."]) });
  const lawIds = new Set(world.laws.map((law) => law.id));
  const assigned = new Set(model.regimes.flatMap((regime) => regime.lawIds ?? []));
  const issues: string[] = [];
  for (const id of assigned) if (!lawIds.has(id)) issues.push(`Regime references unknown law ${id}.`);
  const timeline = regimeTimelineForWorld(world);
  return Object.freeze({ definitions: Object.freeze([...model.regimes]), ...(timeline === undefined ? {} : { timeline }), uncoveredLawIds: Object.freeze([...lawIds].filter((id) => !assigned.has(id))), issues: Object.freeze(issues) });
}

export function addRegimeInterval(model: RegimeModel, interval: RegimeInterval, allowOverlap = false): RegimeModel {
  if (!model.regimes.some((regime) => regime.id === interval.regime)) throw new RangeError(`unknown regime: ${interval.regime}`);
  if (![interval.start, interval.end].every(Number.isFinite) || interval.end <= interval.start) throw new RangeError("regime interval must be finite and non-empty");
  if (interval.confidence !== undefined && (!Number.isFinite(interval.confidence) || interval.confidence < 0 || interval.confidence > 1)) throw new RangeError("regime confidence must be in [0,1]");
  const intervals = [...(model.intervals ?? [])];
  if (!allowOverlap && intervals.some((candidate) => candidate.start < interval.end && interval.start < candidate.end)) throw new RangeError("regime interval overlaps an existing interval");
  intervals.push(interval); intervals.sort((left, right) => left.start - right.start || left.end - right.end);
  return { ...model, intervals };
}
