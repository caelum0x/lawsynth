import type { ParameterDefinition, Uncertainty, UncertaintyModel, WorldDefinition } from "@lawsynth/world-schema";
import { parameterUncertaintyFor, uncertaintySummary, type UncertaintySummary } from "@lawsynth/world-viewer";

export interface UncertaintyCoverage {
  readonly summary: UncertaintySummary;
  readonly parameterCoverage: number;
  readonly parametersWithoutRecord: readonly string[];
  readonly warnings: readonly string[];
}

export function uncertaintyCoverage(world: WorldDefinition): UncertaintyCoverage {
  const parameters = world.parameters ?? [];
  const missing = parameters.filter((parameter) => parameterUncertaintyFor(world, parameter.id) === undefined).map((parameter) => parameter.id);
  const summary = uncertaintySummary(world.uncertainty);
  const warnings: string[] = [];
  if (world.uncertainty === undefined) warnings.push("No uncertainty model is attached to this World.");
  if (missing.length > 0) warnings.push(`${missing.length} parameters have no uncertainty record.`);
  if (summary.counts.trajectory === 0) warnings.push("No trajectory uncertainty bands are recorded.");
  return Object.freeze({ summary, parameterCoverage: parameters.length === 0 ? 1 : (parameters.length - missing.length) / parameters.length, parametersWithoutRecord: Object.freeze(missing), warnings: Object.freeze(warnings) });
}

export function upsertUncertainty(model: UncertaintyModel | undefined, entry: Uncertainty): UncertaintyModel {
  const entries = [...(model?.entries ?? [])];
  const key = uncertaintyKey(entry);
  const index = entries.findIndex((candidate) => uncertaintyKey(candidate) === key);
  if (index < 0) entries.push(entry); else entries[index] = entry;
  return { ...(model ?? {}), entries };
}

function uncertaintyKey(entry: Uncertainty): string {
  if (entry.level === "parameter") return `parameter:${entry.parameter}`;
  if (entry.level === "data") return `data:${entry.variable}`;
  return entry.level;
}

export function parameterRange(parameter: ParameterDefinition, standardError?: number, standardDeviations = 2): readonly [number, number] {
  if (!Number.isFinite(standardDeviations) || standardDeviations <= 0) throw new RangeError("standardDeviations must be positive");
  if (standardError !== undefined && (!Number.isFinite(standardError) || standardError < 0)) throw new RangeError("standardError must be finite and non-negative");
  const spread = standardError === undefined ? 0 : standardError * standardDeviations;
  const lower = Math.max(parameter.bounds?.[0] ?? Number.NEGATIVE_INFINITY, parameter.value - spread);
  const upper = Math.min(parameter.bounds?.[1] ?? Number.POSITIVE_INFINITY, parameter.value + spread);
  return [lower, upper];
}
