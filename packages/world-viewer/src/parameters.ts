import type { ViewerParameter } from "./viewer.js";
export interface ParameterRow extends ViewerParameter { readonly inBounds: boolean; readonly displayValue: string; }
export interface ParameterModel { readonly rows: readonly ParameterRow[]; readonly fixedCount: number; readonly estimatedCount: number; }
export function buildParameterModel(parameters: readonly ViewerParameter[]): ParameterModel { const rows = [...parameters].sort((a,b) => a.id.localeCompare(b.id)).map((parameter) => { const [lower, upper] = parameter.bounds ?? [null, null]; return { ...parameter, inBounds: (lower === null || parameter.value >= lower) && (upper === null || parameter.value <= upper), displayValue: Number.isInteger(parameter.value) ? String(parameter.value) : parameter.value.toPrecision(8) }; }); return { rows, fixedCount: rows.filter((row) => row.fixed === true).length, estimatedCount: rows.filter((row) => row.fixed !== true).length }; }
import type { ParameterDefinition, WorldDefinition } from "@lawsynth/world-schema";

export interface ParameterRow {
  readonly id: string;
  readonly value: number;
  readonly formattedValue: string;
  readonly unit?: string;
  readonly lower?: number;
  readonly upper?: number;
  readonly fixed: boolean;
  readonly description?: string;
}

export interface ParameterOverride {
  readonly id: string;
  readonly value: number;
}

export function formatNumber(value: number, significantDigits = 6): string {
  if (!Number.isFinite(value)) throw new RangeError("parameter value must be finite");
  if (!Number.isInteger(significantDigits) || significantDigits < 1 || significantDigits > 15) {
    throw new RangeError("significantDigits must be between 1 and 15");
  }
  if (value === 0) return "0";
  const magnitude = Math.abs(value);
  return magnitude >= 1e6 || magnitude < 1e-4
    ? value.toExponential(Math.max(0, significantDigits - 1))
    : Number(value.toPrecision(significantDigits)).toString();
}

export function parameterRow(parameter: ParameterDefinition): ParameterRow {
  if (!Number.isFinite(parameter.value)) throw new RangeError(`parameter ${parameter.id} must be finite`);
  const lower = parameter.bounds?.[0] ?? undefined;
  const upper = parameter.bounds?.[1] ?? undefined;
  return Object.freeze({
    id: parameter.id,
    value: parameter.value,
    formattedValue: formatNumber(parameter.value),
    ...(parameter.unit === undefined ? {} : { unit: parameter.unit }),
    ...(lower === undefined ? {} : { lower }),
    ...(upper === undefined ? {} : { upper }),
    fixed: parameter.fixed === true,
    ...(parameter.description === undefined ? {} : { description: parameter.description }),
  });
}

export function parametersForWorld(world: WorldDefinition): readonly ParameterRow[] {
  return Object.freeze((world.parameters ?? []).map(parameterRow));
}

export function validateParameterOverrides(world: WorldDefinition, overrides: readonly ParameterOverride[]): Readonly<Record<string, number>> {
  const definitions = new Map((world.parameters ?? []).map((parameter) => [parameter.id, parameter]));
  const result: Record<string, number> = {};
  for (const override of overrides) {
    if (Object.hasOwn(result, override.id)) throw new RangeError(`duplicate parameter override: ${override.id}`);
    const definition = definitions.get(override.id);
    if (definition === undefined) throw new RangeError(`unknown parameter: ${override.id}`);
    if (definition.fixed) throw new RangeError(`parameter ${override.id} is fixed`);
    if (!Number.isFinite(override.value)) throw new RangeError(`parameter ${override.id} must be finite`);
    const lower = definition.bounds?.[0];
    const upper = definition.bounds?.[1];
    if (lower != null && override.value < lower) throw new RangeError(`parameter ${override.id} is below ${lower}`);
    if (upper != null && override.value > upper) throw new RangeError(`parameter ${override.id} is above ${upper}`);
    result[override.id] = override.value;
  }
  return Object.freeze(result);
}
