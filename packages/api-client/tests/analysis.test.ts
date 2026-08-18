import test from "node:test";
import assert from "node:assert/strict";
import { parseStabilityReport, parseControlledModel, parseDomainRun } from "../dist/index.js";

// Verbatim `lawsynth ... --json` fixtures (see world-schema analysis tests for
// the capture commands). These assert the parsers are reachable from the
// api-client public surface and narrow the engine JSON correctly.

const STABILITY = {
  world: "decay2d.lsworld",
  states: ["x", "y"],
  seeds_total: 25,
  seeds_converged: 25,
  fixed_points: [
    {
      coordinates: [0, 0],
      classification: "stable node",
      inconclusive: false,
      eigenvalues: [
        { re: -2.04166666104394379, im: 0 },
        { re: -1.0102040817416067, im: 0 },
      ],
    },
  ],
};

const CONTROL = {
  source: "forced1d.csv",
  states: ["x"],
  controls: ["u"],
  equations: [
    {
      state: "x",
      residual_sum_squares: 1.95699903604763716e-3,
      terms: [
        { term: "u", coefficient: 9.99393977864784677e-1 },
        { term: "x", coefficient: -4.95801160602859781e-1 },
      ],
    },
  ],
  validation: {
    in_sample: true,
    per_state: [{ state: "x", r_squared: 9.99990552004691668e-1, rmse: 1.96976022511570177e-3 }],
    aggregate_r_squared: 9.99990552004691668e-1,
    aggregate_rmse: 1.96976022511570177e-3,
  },
};

const DOMAIN = {
  preset: "damped-oscillator",
  recovered: true,
  tolerance: 1.00000000000000002e-3,
  laws: ["dv/dt = -0.999987 * x + -0.499985 * v", "dx/dt = 0.999983 * v"],
  recovery: [
    { state: "x", rhs_rmse: 3.72573809999326259e-6, discovered_terms: 1, reference_terms: 1 },
    { state: "v", rhs_rmse: 3.36418375435850972e-6, discovered_terms: 2, reference_terms: 2 },
  ],
};

test("parseStabilityReport narrows a stability report", () => {
  const report = parseStabilityReport(STABILITY);
  assert.equal(report.fixed_points.length, 1);
  assert.equal(report.fixed_points[0].classification, "stable node");
  assert.equal(report.seeds_converged, 25);
  assert.equal(report.fixed_points[0].eigenvalues[0].re, -2.04166666104394379);
});

test("parseControlledModel narrows a validated model", () => {
  const model = parseControlledModel(CONTROL);
  assert.equal(model.equations[0].terms[0].term, "u");
  assert.notEqual(model.validation, null);
  assert.equal(model.validation?.in_sample, true);
  assert.equal(model.validation?.aggregate_rmse, 1.96976022511570177e-3);
});

test("parseControlledModel accepts a null validation block", () => {
  const model = parseControlledModel({ ...CONTROL, validation: null });
  assert.equal(model.validation, null);
});

test("parseDomainRun narrows a round-trip report", () => {
  const report = parseDomainRun(DOMAIN);
  assert.equal(report.preset, "damped-oscillator");
  assert.equal(report.recovered, true);
  assert.equal(report.recovery[1].discovered_terms, 2);
});

test("parsers reject malformed engine JSON", () => {
  assert.throws(() => parseStabilityReport({ world: "w" }));
  assert.throws(() => parseStabilityReport({ ...STABILITY, fixed_points: [{ ...STABILITY.fixed_points[0], classification: "nope" }] }));
  assert.throws(() => parseControlledModel({ source: "s", states: [], controls: [], equations: [] }));
  assert.throws(() => parseDomainRun({ preset: "p", recovered: true, tolerance: 1e-3, laws: [], recovery: [{ state: "x" }] }));
});
