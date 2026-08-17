import type { TrajectoryInput } from "@lawsynth/chart-core";
import {
  evaluateExpression,
  interventionIsActive,
  validateWorld,
  type ContinuousLaw,
  type EvaluationScope,
  type Intervention,
  type ValueIntervention,
  type WorldDefinition,
} from "@lawsynth/world-schema";
import type { LawSynthWasmBindings, WasmSimulationRequest } from "./wasm.js";

export interface EulerConfig {
  readonly start: number;
  readonly end: number;
  readonly step: number;
  readonly initial: Readonly<Record<string, number>>;
  readonly parameters?: Readonly<Record<string, number>>;
  readonly interventions?: readonly Intervention[];
}

/** Upper bound mirrors the WASM runtime default so both paths reject the same request. */
const MAX_SAMPLES = 200_000;

function isContinuous(law: WorldDefinition["laws"][number]): law is ContinuousLaw {
  return law.kind === "continuous" && law.enabled !== false;
}

function numeric(value: number | boolean | string, label: string): number {
  const resolved = typeof value === "boolean" ? (value ? 1 : 0) : Number(value);
  if (!Number.isFinite(resolved)) throw new RangeError(`${label} did not evaluate to a finite number`);
  return resolved;
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
 *
 * It is intentionally small: enough to make the playground respond to parameter
 * overrides and interventions fully offline, using the same
 * `evaluateExpression` the Rust core mirrors. Heavy or stiff solving stays in
 * the service and WASM runtimes; this is the browser-local view-model driver.
 * Output matches `TrajectoryInput` ([sample][variable]) so it flows straight
 * into `charts.ts` and `world-viewer`.
 */
export function forwardEuler(world: WorldDefinition, config: EulerConfig): TrajectoryInput {
  if (![config.start, config.end, config.step].every(Number.isFinite) || config.end <= config.start || config.step <= 0) {
    throw new RangeError("time range must be finite, increasing, and use a positive step");
  }
  const sampleCount = Math.floor((config.end - config.start) / config.step) + 1;
  if (sampleCount > MAX_SAMPLES) throw new RangeError(`forecast would produce ${sampleCount} samples (max ${MAX_SAMPLES})`);

  const continuous = world.laws.filter(isContinuous);
  const stateVars = [...new Set(continuous.map((law) => law.target))];
  if (stateVars.length === 0) throw new RangeError("world has no integrable continuous laws");

  const parameters: Record<string, number> = {};
  for (const parameter of world.parameters ?? []) parameters[parameter.id] = parameter.value;
  for (const [id, value] of Object.entries(config.parameters ?? {})) parameters[id] = value;

  const state: Record<string, number> = {};
  for (const id of stateVars) state[id] = config.initial[id] ?? 0;

  const timeSymbol = world.time.symbol ?? "t";
  const interventions = config.interventions ?? [];
  const times: number[] = [];
  const values: number[][] = [];

  let time = config.start;
  for (let sample = 0; sample < sampleCount; sample += 1) {
    times.push(Number(time.toFixed(6)));
    values.push(stateVars.map((id) => state[id] ?? 0));

    const scope: EvaluationScope = Object.freeze({ ...parameters, ...state, [timeSymbol]: time });
    const derivatives = new Map<string, number>();
    for (const law of continuous) {
      derivatives.set(law.target, numeric(evaluateExpression(law.expression, scope), `law for ${law.target}`));
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

  return Object.freeze({ variables: Object.freeze(stateVars), times: Object.freeze(times), values: Object.freeze(values.map((row) => Object.freeze(row))) });
}

/**
 * A `LawSynthWasmBindings` implementation backed by the local integrator. It
 * satisfies the exact WASM contract (`version`/`simulate`/`validateWorld`) so a
 * `WasmRuntime` — with all of its request-size and sample-count limits — can
 * drive a deterministic simulation without any compiled artifact present.
 */
export function createLocalBindings(version = "playground-euler/0.1.0"): LawSynthWasmBindings {
  return Object.freeze({
    version: () => version,
    simulate: (requestJson: string): string => {
      const request = JSON.parse(requestJson) as WasmSimulationRequest;
      const trajectory = forwardEuler(request.world, {
        start: request.start,
        end: request.end,
        step: request.step,
        initial: request.initial,
        ...(request.parameters === undefined ? {} : { parameters: request.parameters }),
      });
      return JSON.stringify(trajectory);
    },
    validateWorld: (worldJson: string): string => {
      const result = validateWorld(JSON.parse(worldJson));
      return JSON.stringify(result.ok ? { ok: true } : { ok: false, issues: result.issues });
    },
  });
}
