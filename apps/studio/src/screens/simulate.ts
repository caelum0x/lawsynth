import type { TrajectoryInput } from "@lawsynth/chart-core";
import {
  evaluateExpression,
  interventionIsActive,
  type ContinuousLaw,
  type EvaluationScope,
  type Intervention,
  type ValueIntervention,
  type WorldDefinition,
} from "@lawsynth/world-schema";

export interface ForecastConfig {
  readonly horizon: number;
  readonly step: number;
  readonly initialState: Readonly<Record<string, number>>;
  readonly overrides?: Readonly<Record<string, number>>;
  readonly interventions?: readonly Intervention[];
}

const MAX_SAMPLES = 100_000;

function isContinuous(law: WorldDefinition["laws"][number]): law is ContinuousLaw {
  return law.kind === "continuous" && law.enabled !== false;
}

function numeric(value: number | boolean | string, label: string): number {
  const n = typeof value === "boolean" ? (value ? 1 : 0) : Number(value);
  if (!Number.isFinite(n)) throw new RangeError(`${label} did not evaluate to a finite number`);
  return n;
}

function applyValueIntervention(state: Record<string, number>, intervention: ValueIntervention): void {
  const current = state[intervention.target];
  if (current === undefined) return;
  if (intervention.kind === "set") state[intervention.target] = intervention.value;
  else if (intervention.kind === "shift") state[intervention.target] = current + intervention.value;
  else state[intervention.target] = current * intervention.value;
}

/**
 * Deterministic explicit-Euler integrator over a world's continuous laws.
 * It is intentionally small: enough to make the World Lab respond to parameter
 * overrides and interventions offline, using the same `evaluateExpression`
 * the Rust core mirrors. Heavy solving stays in the service.
 */
export function forwardEuler(world: WorldDefinition, config: ForecastConfig): TrajectoryInput {
  if (!Number.isFinite(config.horizon) || config.horizon <= 0) throw new RangeError("horizon must be positive");
  if (!Number.isFinite(config.step) || config.step <= 0 || config.step > config.horizon) {
    throw new RangeError("step must be positive and no larger than the horizon");
  }
  const sampleCount = Math.floor(config.horizon / config.step) + 1;
  if (sampleCount > MAX_SAMPLES) throw new RangeError(`forecast would produce ${sampleCount} samples (max ${MAX_SAMPLES})`);

  const continuous = world.laws.filter(isContinuous);
  const stateVars = [...new Set(continuous.map((law) => law.target))];
  if (stateVars.length === 0) throw new RangeError("world has no integrable continuous laws");

  const parameters: Record<string, number> = {};
  for (const parameter of world.parameters ?? []) parameters[parameter.id] = parameter.value;
  for (const [id, value] of Object.entries(config.overrides ?? {})) parameters[id] = value;

  const state: Record<string, number> = {};
  for (const id of stateVars) state[id] = config.initialState[id] ?? 0;

  const timeSymbol = world.time.symbol ?? "t";
  const interventions = config.interventions ?? [];
  const times: number[] = [];
  const values: number[][] = [];

  let time = 0;
  for (let i = 0; i < sampleCount; i += 1) {
    times.push(Number(time.toFixed(6)));
    values.push(stateVars.map((id) => state[id] ?? 0));

    const scope: EvaluationScope = Object.freeze({ ...parameters, ...state, [timeSymbol]: time });
    const derivatives = new Map<string, number>();
    for (const law of continuous) {
      derivatives.set(law.target, numeric(evaluateExpression(law.expression, scope), `law ${law.id}`));
    }
    for (const id of stateVars) state[id] = (state[id] ?? 0) + config.step * (derivatives.get(id) ?? 0);

    const nextTime = time + config.step;
    for (const intervention of interventions) {
      if (intervention.kind === "set" || intervention.kind === "shift" || intervention.kind === "scale") {
        if (interventionIsActive(intervention, nextTime)) applyValueIntervention(state, intervention);
      }
    }
    time = nextTime;
  }

  return { variables: stateVars, times, values };
}
