import test from "node:test";
import assert from "node:assert/strict";
import {
  parseStabilityReport,
  parseControlledModel,
  parseDomainRun,
  parseBifurcationReport,
  parseSensitivityReport,
  parseEstimateReport,
  parseReductionReport,
  parseLyapunovReport,
  parseBasinReport,
  parseNetworkModel,
  parseMpcResult,
  parseKoopmanReport,
  parseSdeReport,
  parsePdeReport,
  parseInvariantReport,
  parseAnalyzeReport,
  isSkipped,
} from "../dist/index.js";

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

const BIFURCATION = {
  world: "van-der-pol.lsworld",
  states: ["x", "y"],
  parameter: "mu",
  range: { min: -1, max: 1 },
  steps: 21,
  branch_count: 1,
  bifurcations: [
    {
      parameter_value: -2.00000000007485914e-9,
      kind: "hopf",
      branch_id: 0,
      fixed_point: [0, 0],
      eigenvalue: { re: -1.00000000003742957e-9, im: 1 },
    },
  ],
};

const SENSITIVITY = {
  world: "lotka-volterra.lsworld",
  states: ["x", "y"],
  parameters: ["alpha", "beta"],
  final_time: 0.5,
  sensitivities: [
    { state: "x", parameter: "alpha", value: 7.18024197761129912e-1 },
    { state: "x", parameter: "beta", value: -6.68825445802069152e-1 },
    { state: "y", parameter: "alpha", value: 1.38592584652729028e-2 },
    { state: "y", parameter: "beta", value: -1.31930603898465209e-2 },
  ],
};

const ESTIMATE = {
  world: "pendulum.lsworld",
  states: ["omega", "theta"],
  fixed_point: [0, -1.21079836152941656e-14],
  fixed_points_found: 1,
  measured: ["theta"],
  method: "kalman",
  gain: [[-1.14400404642057738e-1], [8.78179475230368101e-1]],
  error_poles: [
    { re: -5.64089737615184106e-1, im: 2.18790932903600144 },
    { re: -5.64089737615183995e-1, im: -2.18790932903600099 },
  ],
  convergent: true,
  covariance: [
    [4.261833187676622, -1.14400404642057738e-1],
    [-1.14400404642057738e-1, 8.78179475230368101e-1],
  ],
};

const REDUCTION = {
  world: "pendulum.lsworld",
  states: ["omega", "theta"],
  fixed_point: [0, -1.21079836152941656e-14],
  measured: null,
  hankel_singular_values: [5.57534663944548292, 5.17456615089844707],
  order: 1,
  error_bound: 1.03491323017968941e1,
  reduced: {
    a: [[-1.2033972395446968e-1]],
    b: [[4.39580802799826253e-1, -1.07174627076214346]],
    c: [[1.04223883190820343], [-5.05578449249294404e-1]],
  },
};

test("parseBifurcationReport narrows a Hopf continuation report", () => {
  const report = parseBifurcationReport(BIFURCATION);
  assert.equal(report.parameter, "mu");
  assert.equal(report.branch_count, 1);
  assert.equal(report.bifurcations[0].kind, "hopf");
  assert.equal(report.bifurcations[0].eigenvalue.im, 1);
});

test("parseSensitivityReport narrows a final-time sensitivity matrix", () => {
  const report = parseSensitivityReport(SENSITIVITY);
  assert.equal(report.final_time, 0.5);
  assert.equal(report.sensitivities.length, 4);
  assert.equal(report.sensitivities[0].parameter, "alpha");
  assert.equal(report.sensitivities[0].value, 7.18024197761129912e-1);
});

test("parseEstimateReport narrows a Kalman estimator with covariance", () => {
  const report = parseEstimateReport(ESTIMATE);
  assert.equal(report.method, "kalman");
  assert.notEqual(report.covariance, null);
  assert.equal(report.covariance?.[0][0], 4.261833187676622);
  assert.equal(report.gain.length, 2);
});

test("parseReductionReport narrows a balanced-truncation report", () => {
  const report = parseReductionReport(REDUCTION);
  assert.equal(report.measured, null);
  assert.equal(report.order, 1);
  assert.equal(report.reduced.a[0][0], -1.2033972395446968e-1);
});

test("parsers reject malformed engine JSON", () => {
  assert.throws(() => parseStabilityReport({ world: "w" }));
  assert.throws(() => parseStabilityReport({ ...STABILITY, fixed_points: [{ ...STABILITY.fixed_points[0], classification: "nope" }] }));
  assert.throws(() => parseControlledModel({ source: "s", states: [], controls: [], equations: [] }));
  assert.throws(() => parseDomainRun({ preset: "p", recovered: true, tolerance: 1e-3, laws: [], recovery: [{ state: "x" }] }));
  assert.throws(() => parseBifurcationReport({ ...BIFURCATION, bifurcations: [{ ...BIFURCATION.bifurcations[0], kind: "nope" }] }));
  assert.throws(() => parseSensitivityReport({ world: "w", states: [], parameters: [] }));
  assert.throws(() => parseEstimateReport({ ...ESTIMATE, method: "luenberger" }));
  assert.throws(() => parseReductionReport({ world: "w", states: [], fixed_point: [], measured: null, hankel_singular_values: [], order: 1, error_bound: 0 }));
});

test("global-dynamics parsers are reachable from the client package", () => {
  const lyap = parseLyapunovReport({
    world: "w", states: ["x", "y"], exponents: [4e-4, -4e-4], largest: 4e-4,
    sum: 0, kaplan_yorke_dimension: 2, integration_time: 90, chaotic: false,
  });
  assert.equal(lyap.chaotic, false);
  assert.equal(lyap.exponents.length, 2);

  const basins = parseBasinReport({
    world: "w", states: ["x"], resolution: 3, total: 3, settled: 2, escaped: 0, undetermined: 1,
    attractors: [{ coordinates: [-1], classification: "stable node", basin_fraction: 1 }],
    grid_labels: ["a0", "undetermined", "a0"],
  });
  assert.equal(basins.attractors.length, 1);
  assert.equal(basins.grid_labels[1], "undetermined");

  const net = parseNetworkModel({
    source: "s", nodes: ["x1", "x2"], adjacency: [[true, false], [true, true]],
    strength: [[1, 0], [0.5, 1]], edges: [{ driver: "x1", target: "x2", strength: 0.5 }],
  });
  assert.equal(net.adjacency[1][0], true);
  assert.equal(net.edges[0].strength, 0.5);

  const mpc = parseMpcResult({
    world: "w", states: ["x"], controls: ["u"], setpoint: [0], final_state: [1e-3],
    final_error_norm: 1e-3, state_trajectory: [[1], [1e-3]], control_trajectory: [[-1], [0]],
  });
  assert.equal(mpc.controls[0], "u");
  assert.ok(mpc.final_error_norm !== null && mpc.final_error_norm < 1e-2);
});

test("global-dynamics parsers reject malformed input", () => {
  assert.throws(() => parseLyapunovReport({ states: ["x"] }));
  assert.throws(() => parseBasinReport({ world: "w", states: ["x"] }));
  assert.throws(() => parseNetworkModel({ nodes: ["x"] }));
  assert.throws(() => parseMpcResult({ states: ["x"] }));
});

// Verbatim `lawsynth {koopman,sde,pde} --json` fixtures (source paths normalized;
// numerics from the debug binary). These assert the discovery-engine parsers are
// reachable from the api-client public surface and narrow the engine JSON.
const KOOPMAN = {
  method: "koopman-dmd",
  source: "koop.csv",
  states: ["x", "y"],
  rank: 2,
  dt: 5.00000000000000028e-2,
  singular_values: [3.42412973113335761, 3.66106644230431433e-1],
  discrete_eigenvalues: [
    { re: 9.51229424500333876e-1, im: 0, modulus: 9.51229424500333876e-1 },
    { re: 9.0483741801937545e-1, im: 0, modulus: 9.0483741801937545e-1 },
  ],
  continuous_eigenvalues: [
    { re: -1.0000000000079925, im: 0 },
    { re: -2.00000000036656589, im: 0 },
  ],
  spectral_radius: 9.51229424500333876e-1,
  stable: true,
};

const SDE = {
  method: "sde-kramers-moyal",
  source: "sde.csv",
  dt: 5.00000000000000028e-2,
  increments: 19999,
  states: [
    {
      state: "x",
      trusted_bins: 8,
      drift: {
        terms: [
          { label: "1", power: 0, coefficient: 1.69498814865994656e-2 },
          { label: "x", power: 1, coefficient: -9.31835524101490953e-1 },
        ],
        residual_sum_squares: 2.23195137552446283e1,
      },
      diffusion: {
        terms: [{ label: "1", power: 0, coefficient: 2.50711161241422897e-1 }],
        residual_sum_squares: 6.57403329307797435e-1,
      },
      bins: [
        { x_center: -1.10328016628529402, drift: 1.08292939561732893, diffusion: 2.58373134849284369e-1, count: 68 },
        { x_center: 1.10013676770666669, drift: -1.01572190893276959, diffusion: 2.24826625914521128e-1, count: 30 },
      ],
    },
  ],
};

const PDE = {
  method: "pde-find",
  source: "pde.csv",
  variable: "u",
  time_snapshots: 60,
  spatial_points: 64,
  dx: 9.81747704246810349e-2,
  dt: 1.00000000000000002e-2,
  interior_points: 3596,
  residual_sum_squares: 1.48656280025936568e-14,
  law: "u_t = -0.049992*u +0.050018*u_xx",
  terms: [
    { label: "1", u_power: 0, derivative_order: 0, coefficient: 0 },
    { label: "u", u_power: 1, derivative_order: 0, coefficient: -4.99916540251027358e-2 },
    { label: "u_xx", u_power: 0, derivative_order: 2, coefficient: 5.00181836393979451e-2 },
  ],
};

test("parseKoopmanReport narrows a DMD spectrum report", () => {
  const report = parseKoopmanReport(KOOPMAN);
  assert.equal(report.method, "koopman-dmd");
  assert.equal(report.rank, 2);
  assert.equal(report.discrete_eigenvalues[0].modulus, 9.51229424500333876e-1);
  assert.equal(report.continuous_eigenvalues[1].re, -2.00000000036656589);
  assert.equal(report.stable, true);
});

test("parseSdeReport narrows drift/diffusion laws and a binned table", () => {
  const report = parseSdeReport(SDE);
  assert.equal(report.method, "sde-kramers-moyal");
  assert.equal(report.increments, 19999);
  assert.equal(report.states[0].trusted_bins, 8);
  assert.equal(report.states[0].drift.terms[1].coefficient, -9.31835524101490953e-1);
  assert.equal(report.states[0].bins[0].count, 68);
});

test("parsePdeReport narrows a PDE-FIND term list", () => {
  const report = parsePdeReport(PDE);
  assert.equal(report.method, "pde-find");
  assert.equal(report.variable, "u");
  assert.equal(report.interior_points, 3596);
  assert.equal(report.terms[2].label, "u_xx");
  assert.equal(report.terms[2].derivative_order, 2);
  assert.equal(report.terms[2].coefficient, 5.00181836393979451e-2);
});

test("discovery-engine parsers reject malformed input", () => {
  assert.throws(() => parseKoopmanReport({ ...KOOPMAN, method: "dmd" }));
  assert.throws(() => parseKoopmanReport({ ...KOOPMAN, discrete_eigenvalues: [{ re: 0, im: 0 }] }));
  assert.throws(() => parseSdeReport({ ...SDE, method: "kramers-moyal" }));
  assert.throws(() => parseSdeReport({ world: "w", states: ["x"] }));
  assert.throws(() => parsePdeReport({ ...PDE, method: "pde" }));
  assert.throws(() => parsePdeReport({ ...PDE, terms: [{ label: "u", u_power: 0, derivative_order: 1.5, coefficient: 1 }] }));
});

// Verbatim `lawsynth {invariants,analyze} --json` fixtures (world paths normalized;
// numerics/labels/combination from the debug binary). These assert the new parsers
// are reachable from the api-client public surface and narrow the engine JSON.
const INVARIANTS = {
  world: "analysis-world.lsworld",
  degree: 2,
  trigonometric: false,
  sample_box: { lo: -1, hi: 1.5 },
  resolution: 5,
  tolerance: 1.00000000000000006e-9,
  basis_labels: ["imbalance", "mid", "imbalance^2", "imbalance*mid", "mid^2"],
  invariants: [
    {
      combination: "0.25·imbalance^2 + 1.00·mid^2",
      coefficients: [-3.31089114135616642e-17, 1.1536867344482037e-17, 2.42535625040950192e-1, -3.15558035916401844e-17, 9.70142500144177578e-1],
      residual: 1.96384182500410133e-15,
      singular_value: 1.42854653545030482e-15,
    },
  ],
};

const ANALYZE = {
  world: "analysis-world.lsworld",
  states: ["imbalance", "mid"],
  stability: {
    world: "analysis-world.lsworld",
    states: ["imbalance", "mid"],
    seeds_total: 25,
    seeds_converged: 25,
    fixed_points: [
      {
        coordinates: [0, 0],
        classification: "center (marginal, inconclusive)",
        inconclusive: true,
        eigenvalues: [
          { re: 0, im: -1.99946666672306694 },
          { re: 0, im: 1.99946666672306672 },
        ],
      },
    ],
  },
  lyapunov: {
    world: "analysis-world.lsworld",
    states: ["imbalance", "mid"],
    exponents: [2.48236620712056284e-4, -2.48236709454393981e-4],
    largest: 2.48236620712056284e-4,
    sum: -8.87423376970397637e-11,
    kaplan_yorke_dimension: 1.99999964250920881,
    integration_time: 89.9999999999991616,
    chaotic: true,
  },
  invariants: INVARIANTS,
};

test("parseInvariantReport narrows a conserved-quantity report", () => {
  const report = parseInvariantReport(INVARIANTS);
  assert.equal(report.degree, 2);
  assert.equal(report.trigonometric, false);
  assert.equal(report.sample_box.hi, 1.5);
  assert.equal(report.basis_labels.length, 5);
  assert.equal(report.invariants[0].combination, "0.25·imbalance^2 + 1.00·mid^2");
  assert.equal(report.invariants[0].coefficients.length, 5);
  assert.equal(report.invariants[0].singular_value, 1.42854653545030482e-15);
});

test("parseAnalyzeReport composes real stability/lyapunov/invariants sub-reports", () => {
  const report = parseAnalyzeReport(ANALYZE);
  assert.equal(report.world, "analysis-world.lsworld");
  assert.equal(report.states.length, 2);
  assert.ok(!isSkipped(report.stability));
  if (!isSkipped(report.stability)) {
    assert.equal(report.stability.fixed_points[0].classification, "center (marginal, inconclusive)");
  }
  assert.ok(!isSkipped(report.lyapunov));
  if (!isSkipped(report.lyapunov)) {
    assert.equal(report.lyapunov.chaotic, true);
    assert.equal(report.lyapunov.largest, 2.48236620712056284e-4);
  }
  assert.ok(!isSkipped(report.invariants));
  if (!isSkipped(report.invariants)) {
    assert.equal(report.invariants.invariants[0].combination, "0.25·imbalance^2 + 1.00·mid^2");
  }
});

test("parseAnalyzeReport narrows a skipped sub-slot", () => {
  const report = parseAnalyzeReport({ ...ANALYZE, lyapunov: { skipped: true, note: "lyapunov integration diverged" } });
  assert.ok(!isSkipped(report.stability));
  assert.ok(isSkipped(report.lyapunov));
  if (isSkipped(report.lyapunov)) {
    assert.equal(report.lyapunov.skipped, true);
    assert.equal(report.lyapunov.note, "lyapunov integration diverged");
  }
});

test("invariants/analyze parsers reject malformed input", () => {
  assert.throws(() => parseInvariantReport({ ...INVARIANTS, invariants: undefined }));
  assert.throws(() => parseInvariantReport({ ...INVARIANTS, degree: 2.5 }));
  assert.throws(() => parseInvariantReport({ ...INVARIANTS, sample_box: { lo: 0 } }));
  assert.throws(() => parseAnalyzeReport(null));
  assert.throws(() => parseAnalyzeReport({ ...ANALYZE, stability: { world: "w", states: [], seeds_total: 1, seeds_converged: 1 } }));
  assert.throws(() => parseAnalyzeReport({ ...ANALYZE, lyapunov: { skipped: true } }));
});
