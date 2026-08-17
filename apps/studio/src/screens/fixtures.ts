import type { CandidateSummary } from "@lawsynth/api-client";
import type { TrajectoryInput } from "@lawsynth/chart-core";
import type { Expression, TrajectoryBand, WorldDefinition } from "@lawsynth/world-schema";

/**
 * Small, self-contained example artifacts so every Studio screen is reachable
 * and interactive without a live service. They are deliberately minimal — a
 * damped harmonic oscillator with two discovered regimes and an uncertainty
 * band — and are only used as seed data when no real world is loaded.
 */

function sym(id: string): Expression {
  return { kind: "symbol", id };
}
function div(left: Expression, right: Expression): Expression {
  return { kind: "binary", operator: "div", left, right };
}
function mul(left: Expression, right: Expression): Expression {
  return { kind: "binary", operator: "mul", left, right };
}
function sub(left: Expression, right: Expression): Expression {
  return { kind: "binary", operator: "sub", left, right };
}
function neg(operand: Expression): Expression {
  return { kind: "unary", operator: "neg", operand };
}

/** Initial state used by the local forecaster; variables carry no initial value in the IR. */
export const FIXTURE_INITIAL_STATE: Readonly<Record<string, number>> = Object.freeze({ x: 1, v: 0 });

function buildBand(): TrajectoryBand {
  const times: number[] = [];
  const lower: number[] = [];
  const median: number[] = [];
  const upper: number[] = [];
  for (let i = 0; i <= 12; i += 1) {
    const t = i;
    const center = Math.cos(t * 0.6) * Math.exp(-0.08 * t);
    const spread = 0.15 + 0.02 * t;
    times.push(t);
    median.push(Number(center.toFixed(4)));
    lower.push(Number((center - spread).toFixed(4)));
    upper.push(Number((center + spread).toFixed(4)));
  }
  return { variable: "x", times, lower, median, upper, confidence: 0.9 };
}

export const FIXTURE_BAND: TrajectoryBand = Object.freeze(buildBand());

export function fixtureWorld(): WorldDefinition {
  return {
    formatVersion: "0.1.0",
    id: "oscillator-demo",
    name: "Damped oscillator (demo)",
    description: "A seeded example world used when no discovery bundle is loaded.",
    time: { kind: "continuous", symbol: "t", unit: "s" },
    variables: [
      { id: "x", name: "position", role: "state", unit: "m", description: "Displacement from equilibrium." },
      { id: "v", name: "velocity", role: "state", unit: "m/s", description: "Rate of change of position." },
    ],
    parameters: [
      { id: "k", value: 4, description: "Spring stiffness.", bounds: [0, 20] },
      { id: "c", value: 0.5, description: "Damping coefficient.", bounds: [0, 5] },
      { id: "m", value: 1, description: "Mass.", fixed: true, bounds: [0.1, 10] },
    ],
    laws: [
      {
        id: "law_x",
        kind: "continuous",
        target: "x",
        expression: sym("v"),
        description: "Position accumulates velocity.",
        enabled: true,
      },
      {
        id: "law_v",
        kind: "continuous",
        target: "v",
        // dv/dt = -(k/m) x - (c/m) v
        expression: sub(neg(mul(div(sym("k"), sym("m")), sym("x"))), mul(div(sym("c"), sym("m")), sym("v"))),
        description: "Newton's second law with linear damping.",
        enabled: true,
      },
    ],
    regimes: {
      regimes: [
        { id: "lightly_damped", name: "Lightly damped", description: "Oscillation dominates over dissipation." },
        { id: "strongly_damped", name: "Strongly damped", description: "Dissipation dominates the response." },
      ],
      intervals: [
        { regime: "lightly_damped", start: 0, end: 6, confidence: 0.82 },
        { regime: "strongly_damped", start: 6, end: 12, confidence: 0.71 },
      ],
      transitions: [{ from: "lightly_damped", to: "strongly_damped", probability: 0.6 }],
      initial: "lightly_damped",
    },
    interventions: [
      { id: "kick", kind: "shift", target: "v", value: 0.8, time: 6, description: "Impulse applied at t=6." },
    ],
    uncertainty: {
      entries: [
        { level: "trajectory", bands: [FIXTURE_BAND] },
        { level: "parameter", parameter: "k", standardError: 0.35, interval: { lower: 3.3, upper: 4.7, confidence: 0.9 } },
      ],
      method: "bootstrap",
      seed: 7,
    },
    provenance: {
      createdAt: "2026-01-01T00:00:00.000Z",
      seed: 7,
      algorithms: [
        { name: "sindy", version: "0.1.0" },
        { name: "regime-hmm", version: "0.1.0" },
      ],
    },
    tags: ["demo", "mechanics"],
  };
}

/** Options controlling the seeded shock injected into a comparison dataset. */
export interface ShockOptions {
  readonly variableIndex?: number;
  readonly startFraction?: number;
  readonly magnitude?: number;
  readonly decay?: number;
}

/**
 * Builds a deterministic "new data" comparison dataset from a clean baseline
 * trajectory by injecting a localized shock into one state: a step that appears
 * partway through the window and decays exponentially. Because the baseline is
 * typically the model's own prediction, the resulting residuals are ~zero
 * everywhere except across the shock, giving Monitor an unambiguous anomaly to
 * flag and an in-control → drift transition to verdict on. Pure: the input is
 * copied, never mutated.
 */
export function shockDataset(baseline: TrajectoryInput, options: ShockOptions = {}): TrajectoryInput {
  const rows = baseline.times.length;
  if (rows === 0 || baseline.variables.length === 0) return baseline;
  const target = Math.min(Math.max(0, options.variableIndex ?? 0), baseline.variables.length - 1);
  const start = Math.floor(rows * Math.min(0.95, Math.max(0, options.startFraction ?? 0.6)));
  const magnitude = options.magnitude ?? 0.9;
  const decay = options.decay ?? 0.35;
  const values = baseline.values.map((row, index) => {
    const copy = row.slice();
    if (index >= start) {
      const elapsed = index - start;
      copy[target] = (copy[target] ?? 0) + magnitude * Math.exp(-decay * elapsed);
    }
    return copy;
  });
  return { variables: baseline.variables.slice(), times: baseline.times.slice(), values };
}

export function fixtureCandidates(runId = "run-demo"): readonly CandidateSummary[] {
  return Object.freeze([
    { id: "cand-1", run_id: runId, score: 0.982, equation: "dv/dt = -(k/m) x - (c/m) v", world_id: "oscillator-demo" },
    { id: "cand-2", run_id: runId, score: 0.944, equation: "dv/dt = -(k/m) x - c v^2" },
    { id: "cand-3", run_id: runId, score: 0.901, equation: "dv/dt = -k sin(x) - c v" },
    { id: "cand-4", run_id: runId, score: 0.786, equation: "dv/dt = -k x" },
  ]);
}
