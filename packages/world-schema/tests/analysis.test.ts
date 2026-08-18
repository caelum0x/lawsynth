import {
  parseBasinReport,
  parseBifurcationReport,
  parseControlledModel,
  parseDomainRun,
  parseEstimateReport,
  parseKoopmanReport,
  parseLyapunovReport,
  parseMpcResult,
  parseNetworkModel,
  parsePdeReport,
  parseReductionReport,
  parseSdeReport,
  parseSensitivityReport,
  parseStabilityReport,
  validateStabilityReport,
} from "../src/analysis.js";
import { equal, ok, throws } from "./test-support.js";

// All fixtures below are VERBATIM output of the real `lawsynth` CLI (--json),
// captured from the debug binary against synthesized datasets. See
// crates/lawsynth-cli/src/{stability,control,domains}.rs render_json.

// `lawsynth stability decay2d.lsworld --box -1:1,-1:1 --json` (stable node).
const STABILITY_STABLE_NODE = `{
  "world": "decay2d.lsworld",
  "states": ["x", "y"],
  "seeds_total": 25,
  "seeds_converged": 25,
  "fixed_points": [
    {
      "coordinates": [0.00000000000000000e0, 0.00000000000000000e0],
      "classification": "stable node",
      "inconclusive": false,
      "eigenvalues": [{"re": -2.04166666104394379e0, "im": 0.00000000000000000e0}, {"re": -1.01020408174160670e0, "im": 0.00000000000000000e0}]
    }
  ]
}`;

// `lawsynth stability osc.lsworld --box -0.5:0.5,-0.5:0.5 --json` (center).
const STABILITY_CENTER = `{
  "world": "osc.lsworld",
  "states": ["v", "x"],
  "seeds_total": 25,
  "seeds_converged": 25,
  "fixed_points": [
    {
      "coordinates": [0.00000000000000000e0, 0.00000000000000000e0],
      "classification": "center (marginal, inconclusive)",
      "inconclusive": true,
      "eigenvalues": [{"re": 0.00000000000000000e0, "im": -9.99984274058213995e-1}, {"re": 0.00000000000000000e0, "im": 9.99984274058213329e-1}]
    }
  ]
}`;

// `lawsynth control forced1d.csv --time t --state x --control u --degree 1 --validate --json`.
const CONTROL_VALIDATED = `{
  "source": "forced1d.csv",
  "states": ["x"],
  "controls": ["u"],
  "equations": [
    {
      "state": "x",
      "residual_sum_squares": 1.95699903604763716e-3,
      "terms": [{"term": "u", "coefficient": 9.99393977864784677e-1}, {"term": "x", "coefficient": -4.95801160602859781e-1}]
    }
  ],
  "validation": {
    "in_sample": true,
    "per_state": [{"state": "x", "r_squared": 9.99990552004691668e-1, "rmse": 1.96976022511570177e-3}],
    "aggregate_r_squared": 9.99990552004691668e-1,
    "aggregate_rmse": 1.96976022511570177e-3
  }
}`;

// Same command without `--validate`: validation is null (hand-derived from the
// render_json None branch, which emits `"validation": null`).
const CONTROL_UNVALIDATED = `{
  "source": "forced1d.csv",
  "states": ["x"],
  "controls": ["u"],
  "equations": [
    {
      "state": "x",
      "residual_sum_squares": 1.95699903604763716e-3,
      "terms": [{"term": "u", "coefficient": 9.99393977864784677e-1}, {"term": "x", "coefficient": -4.95801160602859781e-1}]
    }
  ],
  "validation": null
}`;

// `lawsynth domains run damped-oscillator --json`.
const DOMAIN_RUN = `{
  "preset": "damped-oscillator",
  "recovered": true,
  "tolerance": 1.00000000000000002e-3,
  "laws": [
    "dv/dt = -0.999987 * x + -0.499985 * v",
    "dx/dt = 0.999983 * v"
  ],
  "recovery": [
    {"state": "x", "rhs_rmse": 3.72573809999326259e-6, "discovered_terms": 1, "reference_terms": 1},
    {"state": "v", "rhs_rmse": 3.36418375435850972e-6, "discovered_terms": 2, "reference_terms": 2}
  ]
}`;

// `lawsynth bifurcation van-der-pol.lsworld --parameter mu --range -1:1 --box
// -3:3,-3:3 --steps 21 --json` — the Van der Pol origin loses stability at mu=0
// (a Hopf). Captured verbatim from the debug binary; only the "world" path string
// was normalized to a clean bundle name (all numerics are the engine's output).
const BIFURCATION_HOPF = `{
  "world": "van-der-pol.lsworld",
  "states": ["x", "y"],
  "parameter": "mu",
  "range": {"min": -1.00000000000000000e0, "max": 1.00000000000000000e0},
  "steps": 21,
  "branch_count": 1,
  "bifurcations": [
    {
      "parameter_value": -2.00000000007485914e-9,
      "kind": "hopf",
      "branch_id": 0,
      "fixed_point": [0.00000000000000000e0, 0.00000000000000000e0],
      "eigenvalue": {"re": -1.00000000003742957e-9, "im": 1.00000000000000000e0}
    }
  ]
}`;

// `lawsynth sensitivity lotka-volterra.lsworld --parameters alpha,beta --initial
// x=1 --initial y=1 --dt 0.01 --steps 50 --json` (world path normalized).
const SENSITIVITY = `{
  "world": "lotka-volterra.lsworld",
  "states": ["x", "y"],
  "parameters": ["alpha", "beta"],
  "final_time": 5.00000000000000000e-1,
  "sensitivities": [
    {"state": "x", "parameter": "alpha", "value": 7.18024197761129912e-1},
    {"state": "x", "parameter": "beta", "value": -6.68825445802069152e-1},
    {"state": "y", "parameter": "alpha", "value": 1.38592584652729028e-2},
    {"state": "y", "parameter": "beta", "value": -1.31930603898465209e-2}
  ]
}`;

// `lawsynth estimate pendulum.lsworld --box -0.5:0.5,-0.5:0.5 --measure theta
// --poles -2,-3 --json` — pole placement, so covariance is null (world path
// normalized; numerics verbatim).
const ESTIMATE_POLES = `{
  "world": "pendulum.lsworld",
  "states": ["omega", "theta"],
  "fixed_point": [0.00000000000000000e0, -1.21079836152941656e-14],
  "fixed_points_found": 1,
  "measured": ["theta"],
  "method": "pole_placement",
  "gain": [[-1.87500000000000000e-1], [4.75000000000000000e0]],
  "error_poles": [{"re": -2.99999999999999956e0, "im": 0.00000000000000000e0}, {"re": -2.00000000000000044e0, "im": 0.00000000000000000e0}],
  "convergent": true,
  "covariance": null
}`;

// `lawsynth estimate pendulum.lsworld --box -0.5:0.5,-0.5:0.5 --measure theta
// --kalman --json` — the Kalman branch emits a non-null covariance matrix.
const ESTIMATE_KALMAN = `{
  "world": "pendulum.lsworld",
  "states": ["omega", "theta"],
  "fixed_point": [0.00000000000000000e0, -1.21079836152941656e-14],
  "fixed_points_found": 1,
  "measured": ["theta"],
  "method": "kalman",
  "gain": [[-1.14400404642057738e-1], [8.78179475230368101e-1]],
  "error_poles": [{"re": -5.64089737615184106e-1, "im": 2.18790932903600144e0}, {"re": -5.64089737615183995e-1, "im": -2.18790932903600099e0}],
  "convergent": true,
  "covariance": [[4.26183318767662200e0, -1.14400404642057738e-1], [-1.14400404642057738e-1, 8.78179475230368101e-1]]
}`;

// `lawsynth reduce pendulum.lsworld --box -0.5:0.5,-0.5:0.5 --order 1 --json` —
// no --measure, so "measured" is null and C defaults to I (world path normalized).
const REDUCTION = `{
  "world": "pendulum.lsworld",
  "states": ["omega", "theta"],
  "fixed_point": [0.00000000000000000e0, -1.21079836152941656e-14],
  "measured": null,
  "hankel_singular_values": [5.57534663944548292e0, 5.17456615089844707e0],
  "order": 1,
  "error_bound": 1.03491323017968941e1,
  "reduced": {
    "a": [[-1.20339723954469680e-1]],
    "b": [[4.39580802799826253e-1, -1.07174627076214346e0]],
    "c": [[1.04223883190820343e0], [-5.05578449249294404e-1]]
  }
}`;

// `lawsynth reduce pendulum.lsworld --box -0.5:0.5,-0.5:0.5 --tolerance 0.4
// --measure theta --json` — --measure selects C, so "measured" is a string array.
const REDUCTION_MEASURED = `{
  "world": "pendulum.lsworld",
  "states": ["omega", "theta"],
  "fixed_point": [0.00000000000000000e0, -1.21079836152941656e-14],
  "measured": ["theta"],
  "hankel_singular_values": [2.21592595653845903e0, 2.16801687150702760e0],
  "order": 2,
  "error_bound": -0.00000000000000000e0,
  "reduced": {
    "a": [[-1.78191631255412930e-1, 2.32365196620965264e0], [-2.14627854005638996e0, -7.18083687445867369e-2]],
    "b": [[2.27703601385121773e-1, 8.58993592403251238e-1], [3.62637167908169578e-1, -4.24096444643041415e-1]],
    "c": [[8.88661308864968524e-1, -5.57999560848684850e-1]]
  }
}`;

// `lawsynth koopman koop.csv --state x,y --time time --json` on a linear-decay
// dataset (x'=-x, y'=-2y). Captured verbatim from the debug binary; only the
// "source" path string was normalized to a clean name (all numerics verbatim).
const KOOPMAN = `{
  "method": "koopman-dmd",
  "source": "koop.csv",
  "states": ["x", "y"],
  "rank": 2,
  "dt": 5.00000000000000028e-2,
  "singular_values": [3.42412973113335761e0, 3.66106644230431433e-1],
  "discrete_eigenvalues": [{"re": 9.51229424500333876e-1, "im": 0.00000000000000000e0, "modulus": 9.51229424500333876e-1}, {"re": 9.04837418019375450e-1, "im": 0.00000000000000000e0, "modulus": 9.04837418019375450e-1}],
  "continuous_eigenvalues": [{"re": -1.00000000000799250e0, "im": 0.00000000000000000e0}, {"re": -2.00000000036656589e0, "im": 0.00000000000000000e0}],
  "spectral_radius": 9.51229424500333876e-1,
  "stable": true
}`;

// `lawsynth sde sde.csv --state x --time time --bins 8 --json` on an
// Ornstein–Uhlenbeck path (dX = -X dt + 0.5 dW). Captured verbatim from the
// debug binary; only the "source" path string was normalized (numerics verbatim).
const SDE = `{
  "method": "sde-kramers-moyal",
  "source": "sde.csv",
  "dt": 5.00000000000000028e-2,
  "increments": 19999,
  "states": [
    {
      "state": "x",
      "trusted_bins": 8,
      "drift": {"terms": [{"label": "1", "power": 0, "coefficient": 1.69498814865994656e-2}, {"label": "x", "power": 1, "coefficient": -9.31835524101490953e-1}, {"label": "x^2", "power": 2, "coefficient": -2.12975865910596279e-1}, {"label": "x^3", "power": 3, "coefficient": -1.52643134342554637e-1}], "residual_sum_squares": 2.23195137552446283e1},
      "diffusion": {"terms": [{"label": "1", "power": 0, "coefficient": 2.50711161241422897e-1}, {"label": "x", "power": 1, "coefficient": 1.49374220289966546e-2}, {"label": "x^2", "power": 2, "coefficient": 4.17055916620023073e-2}, {"label": "x^3", "power": 3, "coefficient": -1.23121897480200516e-2}], "residual_sum_squares": 6.57403329307797435e-1},
      "bins": [{"x_center": -1.10328016628529402e0, "drift": 1.08292939561732893e0, "diffusion": 2.58373134849284369e-1, "count": 68}, {"x_center": -7.88810664666158123e-1, "drift": 6.49277992405561610e-1, "diffusion": 2.82760572702799806e-1, "count": 656}, {"x_center": -4.64969450051377575e-1, "drift": 4.37726142558743425e-1, "diffusion": 2.50602777787971076e-1, "count": 2978}, {"x_center": -1.56798329500426437e-1, "drift": 1.54634039845952864e-1, "diffusion": 2.52369588617901486e-1, "count": 6325}, {"x_center": 1.52218175850468818e-1, "drift": -1.40197793650390562e-1, "diffusion": 2.49814792001835595e-1, "count": 6402}, {"x_center": 4.63272227717627161e-1, "drift": -4.39114484668472249e-1, "diffusion": 2.69923827217231238e-1, "count": 3007}, {"x_center": 7.59661785772231868e-1, "drift": -1.00727494333196410e0, "diffusion": 2.83716993160374575e-1, "count": 533}, {"x_center": 1.10013676770666669e0, "drift": -1.01572190893276959e0, "diffusion": 2.24826625914521128e-1, "count": 30}]
    }
  ]
}`;

// `lawsynth pde pde.csv --dx 0.0981... --dt 0.01 --json` on a heat-equation grid
// (u_t = 0.05 u_xx). Captured verbatim from the debug binary; only the "source"
// path string was normalized to a clean name (all numerics verbatim).
const PDE = `{
  "method": "pde-find",
  "source": "pde.csv",
  "variable": "u",
  "time_snapshots": 60,
  "spatial_points": 64,
  "dx": 9.81747704246810349e-2,
  "dt": 1.00000000000000002e-2,
  "interior_points": 3596,
  "residual_sum_squares": 1.48656280025936568e-14,
  "law": "u_t = -0.049992*u +0.050018*u_xx",
  "terms": [{"label": "1", "u_power": 0, "derivative_order": 0, "coefficient": 0.00000000000000000e0}, {"label": "u", "u_power": 1, "derivative_order": 0, "coefficient": -4.99916540251027358e-2}, {"label": "u^2", "u_power": 2, "derivative_order": 0, "coefficient": 0.00000000000000000e0}, {"label": "u_x", "u_power": 0, "derivative_order": 1, "coefficient": 0.00000000000000000e0}, {"label": "u*u_x", "u_power": 1, "derivative_order": 1, "coefficient": 0.00000000000000000e0}, {"label": "u^2*u_x", "u_power": 2, "derivative_order": 1, "coefficient": 0.00000000000000000e0}, {"label": "u_xx", "u_power": 0, "derivative_order": 2, "coefficient": 5.00181836393979451e-2}, {"label": "u*u_xx", "u_power": 1, "derivative_order": 2, "coefficient": 0.00000000000000000e0}, {"label": "u^2*u_xx", "u_power": 2, "derivative_order": 2, "coefficient": 0.00000000000000000e0}]
}`;

export function runAnalysisTests(): void {
  // --- stability: stable node ---
  const stable = parseStabilityReport(JSON.parse(STABILITY_STABLE_NODE));
  equal(stable.world, "decay2d.lsworld");
  equal(stable.states.length, 2);
  equal(stable.states[0], "x");
  equal(stable.seeds_total, 25);
  equal(stable.seeds_converged, 25);
  equal(stable.fixed_points.length, 1);
  const node = stable.fixed_points[0]!;
  equal(node.classification, "stable node");
  equal(node.inconclusive, false);
  equal(node.coordinates.length, 2);
  equal(node.coordinates[0], 0);
  equal(node.eigenvalues.length, 2);
  equal(node.eigenvalues[0]!.re, -2.04166666104394379);
  equal(node.eigenvalues[0]!.im, 0);

  // --- stability: center carries the exact "(... inconclusive)" verdict ---
  const center = parseStabilityReport(JSON.parse(STABILITY_CENTER));
  const centerPoint = center.fixed_points[0]!;
  equal(centerPoint.classification, "center (marginal, inconclusive)");
  equal(centerPoint.inconclusive, true);
  equal(centerPoint.eigenvalues[1]!.im, 9.99984274058213329e-1);

  // --- control: with in-sample validation ---
  const model = parseControlledModel(JSON.parse(CONTROL_VALIDATED));
  equal(model.source, "forced1d.csv");
  equal(model.states[0], "x");
  equal(model.controls[0], "u");
  equal(model.equations.length, 1);
  const equation = model.equations[0]!;
  equal(equation.state, "x");
  equal(equation.residual_sum_squares, 1.95699903604763716e-3);
  equal(equation.terms.length, 2);
  equal(equation.terms[0]!.term, "u");
  equal(equation.terms[1]!.coefficient, -4.95801160602859781e-1);
  ok(model.validation !== null, "validation should be present");
  equal(model.validation!.in_sample, true);
  equal(model.validation!.per_state[0]!.state, "x");
  equal(model.validation!.aggregate_r_squared, 9.99990552004691668e-1);

  // --- control: without validation (null) ---
  const bare = parseControlledModel(JSON.parse(CONTROL_UNVALIDATED));
  equal(bare.validation, null);
  equal(bare.equations[0]!.terms.length, 2);

  // --- domains run ---
  const domain = parseDomainRun(JSON.parse(DOMAIN_RUN));
  equal(domain.preset, "damped-oscillator");
  equal(domain.recovered, true);
  equal(domain.tolerance, 1.00000000000000002e-3);
  equal(domain.laws.length, 2);
  equal(domain.laws[0], "dv/dt = -0.999987 * x + -0.499985 * v");
  equal(domain.recovery.length, 2);
  equal(domain.recovery[1]!.state, "v");
  equal(domain.recovery[1]!.discovered_terms, 2);
  equal(domain.recovery[0]!.rhs_rmse, 3.72573809999326259e-6);

  // --- error paths: malformed input is rejected ---
  throws(() => parseStabilityReport(null), "null is not a report");
  throws(() => parseStabilityReport("{}"), "a JSON string is not a parsed object");
  throws(() => parseStabilityReport({ world: "w", states: ["x"], seeds_total: 1, seeds_converged: 1 }), "missing fixed_points");
  // Unknown classification verdict must be rejected.
  throws(
    () =>
      parseStabilityReport({
        world: "w",
        states: ["x"],
        seeds_total: 1,
        seeds_converged: 1,
        fixed_points: [{ coordinates: [0], classification: "attractor", inconclusive: false, eigenvalues: [] }],
      }),
    "unknown classification is rejected",
  );
  // seeds must be non-negative integers, not floats.
  throws(
    () => parseStabilityReport({ world: "w", states: [], seeds_total: 1.5, seeds_converged: 0, fixed_points: [] }),
    "fractional seed count is rejected",
  );
  throws(() => parseControlledModel({ source: "s", states: [], controls: [], equations: [] }), "missing validation key");
  throws(() => parseControlledModel({ source: "s", states: [], controls: [], equations: [{ state: "x" }], validation: null }), "equation missing residual/terms");
  throws(() => parseDomainRun({ preset: "p", recovered: true, tolerance: 1e-3, laws: [1], recovery: [] }), "law must be a string");
  throws(() => parseDomainRun({ preset: "p", recovered: "yes", tolerance: 1e-3, laws: [], recovery: [] }), "recovered must be a boolean");

  // --- bifurcation: a real Hopf on the Van der Pol origin ---
  const bifurcation = parseBifurcationReport(JSON.parse(BIFURCATION_HOPF));
  equal(bifurcation.world, "van-der-pol.lsworld");
  equal(bifurcation.parameter, "mu");
  equal(bifurcation.range.min, -1);
  equal(bifurcation.range.max, 1);
  equal(bifurcation.steps, 21);
  equal(bifurcation.branch_count, 1);
  equal(bifurcation.bifurcations.length, 1);
  const hopf = bifurcation.bifurcations[0]!;
  equal(hopf.kind, "hopf");
  equal(hopf.branch_id, 0);
  equal(hopf.parameter_value, -2.00000000007485914e-9);
  equal(hopf.fixed_point.length, 2);
  equal(hopf.eigenvalue.im, 1);

  // --- sensitivity: dx_i/dtheta_j at the final time ---
  const sensitivity = parseSensitivityReport(JSON.parse(SENSITIVITY));
  equal(sensitivity.states.length, 2);
  equal(sensitivity.parameters[0], "alpha");
  equal(sensitivity.final_time, 0.5);
  equal(sensitivity.sensitivities.length, 4);
  equal(sensitivity.sensitivities[0]!.state, "x");
  equal(sensitivity.sensitivities[0]!.parameter, "alpha");
  equal(sensitivity.sensitivities[0]!.value, 7.18024197761129912e-1);
  equal(sensitivity.sensitivities[3]!.parameter, "beta");

  // --- estimate: pole placement (covariance null) ---
  const estimate = parseEstimateReport(JSON.parse(ESTIMATE_POLES));
  equal(estimate.method, "pole_placement");
  equal(estimate.states[0], "omega");
  equal(estimate.fixed_points_found, 1);
  equal(estimate.measured[0], "theta");
  equal(estimate.gain.length, 2);
  equal(estimate.gain[0]![0], -1.875e-1);
  equal(estimate.error_poles.length, 2);
  equal(estimate.error_poles[0]!.re, -2.99999999999999956e0);
  equal(estimate.convergent, true);
  equal(estimate.covariance, null);

  // --- estimate: Kalman branch carries a non-null covariance matrix ---
  const kalman = parseEstimateReport(JSON.parse(ESTIMATE_KALMAN));
  equal(kalman.method, "kalman");
  ok(kalman.covariance !== null, "kalman estimate has a covariance matrix");
  equal(kalman.covariance!.length, 2);
  equal(kalman.covariance![0]!.length, 2);
  equal(kalman.covariance![0]![0], 4.261833187676622e0);
  equal(kalman.error_poles[1]!.im, -2.18790932903600099e0);

  // --- reduce: order truncation with C = I (measured null) ---
  const reduction = parseReductionReport(JSON.parse(REDUCTION));
  equal(reduction.measured, null);
  equal(reduction.hankel_singular_values.length, 2);
  equal(reduction.order, 1);
  equal(reduction.error_bound, 1.03491323017968941e1);
  equal(reduction.reduced.a.length, 1);
  equal(reduction.reduced.a[0]![0], -1.2033972395446968e-1);
  equal(reduction.reduced.b[0]!.length, 2);
  equal(reduction.reduced.c.length, 2);

  // --- reduce: --measure selects C, so measured is a string array ---
  const reductionMeasured = parseReductionReport(JSON.parse(REDUCTION_MEASURED));
  ok(reductionMeasured.measured !== null, "measured is present when --measure is given");
  equal(reductionMeasured.measured![0], "theta");
  equal(reductionMeasured.order, 2);

  // --- error paths: malformed input is rejected per shape ---
  throws(() => parseBifurcationReport(null), "null is not a bifurcation report");
  // Unknown bifurcation kind token must be rejected.
  throws(
    () =>
      parseBifurcationReport({
        world: "w",
        states: ["x"],
        parameter: "p",
        range: { min: 0, max: 1 },
        steps: 2,
        branch_count: 1,
        bifurcations: [{ parameter_value: 0, kind: "transcritical", branch_id: 0, fixed_point: [0], eigenvalue: { re: 0, im: 0 } }],
      }),
    "unknown bifurcation kind is rejected",
  );
  // range must be an object with numeric min/max.
  throws(
    () => parseBifurcationReport({ world: "w", states: [], parameter: "p", range: { min: 0 }, steps: 1, branch_count: 0, bifurcations: [] }),
    "range missing max is rejected",
  );
  throws(() => parseSensitivityReport({ world: "w", states: [], parameters: [] }), "missing final_time/sensitivities");
  throws(
    () => parseSensitivityReport({ world: "w", states: [], parameters: [], final_time: 1, sensitivities: [{ state: "x", parameter: "a" }] }),
    "sensitivity entry missing value",
  );
  // Unknown observer method token must be rejected.
  throws(
    () =>
      parseEstimateReport({
        world: "w",
        states: ["x"],
        fixed_point: [0],
        fixed_points_found: 1,
        measured: ["x"],
        method: "luenberger",
        gain: [[1]],
        error_poles: [{ re: -1, im: 0 }],
        convergent: true,
        covariance: null,
      }),
    "unknown observer method is rejected",
  );
  // covariance is required (null when not --kalman); omitting it is an error.
  throws(
    () => parseEstimateReport({ world: "w", states: ["x"], fixed_point: [0], fixed_points_found: 1, measured: ["x"], method: "kalman", gain: [[1]], error_poles: [], convergent: true }),
    "missing covariance key is rejected",
  );
  throws(() => parseReductionReport({ world: "w", states: [], fixed_point: [], measured: null, hankel_singular_values: [], order: 1, error_bound: 0 }), "missing reduced block");
  throws(
    () => parseReductionReport({ world: "w", states: [], fixed_point: [], measured: null, hankel_singular_values: [], order: 1.5, error_bound: 0, reduced: { a: [[1]], b: [[1]], c: [[1]] } }),
    "fractional reduced order is rejected",
  );

  // --- global dynamics: lyapunov / basins / network / mpc ---
  // Fixtures are shaped exactly like the `lawsynth {lyapunov,basins,network,mpc}
  // --json` render fns in crates/lawsynth-cli/src/*.rs.
  const lyap = parseLyapunovReport(
    JSON.parse(`{
      "world": "oscillator.lsworld", "states": ["x", "y"],
      "exponents": [4.0e-4, -4.0e-4], "largest": 4.0e-4, "sum": 0.0,
      "kaplan_yorke_dimension": 2.0, "integration_time": 90.0, "chaotic": false
    }`),
  );
  equal(lyap.exponents.length, 2);
  equal(lyap.largest, 4.0e-4);
  equal(lyap.sum, 0.0);
  equal(lyap.chaotic, false);

  const basins = parseBasinReport(
    JSON.parse(`{
      "world": "bistable.lsworld", "states": ["x"],
      "resolution": 5, "total": 5, "settled": 4, "escaped": 0, "undetermined": 1,
      "attractors": [
        {"coordinates": [-1.0], "classification": "stable node", "basin_fraction": 0.5},
        {"coordinates": [1.0], "classification": "stable node", "basin_fraction": 0.5}
      ],
      "grid_labels": ["a0", "a0", "undetermined", "a1", "a1"]
    }`),
  );
  equal(basins.attractors.length, 2);
  equal(basins.attractors[0]?.classification, "stable node");
  equal(basins.grid_labels[2], "undetermined");
  equal(basins.settled, 4);

  const net = parseNetworkModel(
    JSON.parse(`{
      "source": "chain.csv", "nodes": ["x1", "x2", "x3"],
      "adjacency": [[true, false, false], [true, true, false], [false, true, true]],
      "strength": [[1.0, 0.0, 0.0], [0.5, 1.0, 0.0], [0.0, 0.5, 1.0]],
      "edges": [
        {"driver": "x1", "target": "x2", "strength": 0.5},
        {"driver": "x2", "target": "x3", "strength": 0.5}
      ]
    }`),
  );
  equal(net.nodes.length, 3);
  equal(net.adjacency[1]?.[0], true); // x1 -> x2
  equal(net.adjacency[2]?.[0], false); // no x1 -> x3
  equal(net.edges[0]?.driver, "x1");

  const mpc = parseMpcResult(
    JSON.parse(`{
      "world": "double-integrator.lsworld", "states": ["x", "v"], "controls": ["u"],
      "setpoint": [0.0, 0.0], "final_state": [8.0e-4, -3.0e-4], "final_error_norm": 8.5e-4,
      "state_trajectory": [[1.0, 0.0], [8.0e-4, -3.0e-4]], "control_trajectory": [[-1.2], [-0.01]]
    }`),
  );
  equal(mpc.controls[0], "u");
  ok(mpc.final_error_norm !== null && mpc.final_error_norm < 1e-2, "mpc regulated");
  equal(mpc.state_trajectory.length, 2);
  // final_error_norm may be null when unavailable.
  const mpcNull = parseMpcResult(
    JSON.parse(`{
      "world": "w", "states": ["x"], "controls": ["u"], "setpoint": [0.0],
      "final_state": [0.1], "final_error_norm": null,
      "state_trajectory": [[0.1]], "control_trajectory": [[0.0]]
    }`),
  );
  equal(mpcNull.final_error_norm, null);

  // malformed inputs are rejected
  throws(() => parseLyapunovReport({ states: ["x"] }), "lyapunov missing exponents");
  throws(() => parseBasinReport({ world: "w", states: ["x"] }), "basins missing attractors");
  throws(() => parseNetworkModel({ nodes: ["x"] }), "network missing edges");
  throws(() => parseMpcResult({ states: ["x"] }), "mpc missing controls");

  // --- koopman: a DMD linear-operator spectrum ---
  const koopman = parseKoopmanReport(JSON.parse(KOOPMAN));
  equal(koopman.method, "koopman-dmd");
  equal(koopman.source, "koop.csv");
  equal(koopman.states.length, 2);
  equal(koopman.states[1], "y");
  equal(koopman.rank, 2);
  equal(koopman.dt, 5.00000000000000028e-2);
  equal(koopman.singular_values.length, 2);
  equal(koopman.singular_values[0], 3.42412973113335761e0);
  equal(koopman.discrete_eigenvalues.length, 2);
  equal(koopman.discrete_eigenvalues[0]!.re, 9.51229424500333876e-1);
  equal(koopman.discrete_eigenvalues[0]!.modulus, 9.51229424500333876e-1);
  equal(koopman.continuous_eigenvalues.length, 2);
  equal(koopman.continuous_eigenvalues[1]!.re, -2.00000000036656589e0);
  equal(koopman.spectral_radius, 9.51229424500333876e-1);
  equal(koopman.stable, true);

  // --- sde: drift/diffusion laws + binned Kramers–Moyal table ---
  const sde = parseSdeReport(JSON.parse(SDE));
  equal(sde.method, "sde-kramers-moyal");
  equal(sde.source, "sde.csv");
  equal(sde.dt, 5.00000000000000028e-2);
  equal(sde.increments, 19999);
  equal(sde.states.length, 1);
  const sdeState = sde.states[0]!;
  equal(sdeState.state, "x");
  equal(sdeState.trusted_bins, 8);
  equal(sdeState.drift.terms.length, 4);
  equal(sdeState.drift.terms[1]!.label, "x");
  equal(sdeState.drift.terms[1]!.power, 1);
  equal(sdeState.drift.terms[1]!.coefficient, -9.31835524101490953e-1);
  equal(sdeState.drift.residual_sum_squares, 2.23195137552446283e1);
  equal(sdeState.diffusion.terms[0]!.coefficient, 2.50711161241422897e-1);
  equal(sdeState.bins.length, 8);
  equal(sdeState.bins[0]!.x_center, -1.10328016628529402e0);
  equal(sdeState.bins[0]!.count, 68);
  equal(sdeState.bins[3]!.count, 6325);

  // --- pde: PDE-FIND u_t term list (all library terms, active + thresholded) ---
  const pde = parsePdeReport(JSON.parse(PDE));
  equal(pde.method, "pde-find");
  equal(pde.source, "pde.csv");
  equal(pde.variable, "u");
  equal(pde.time_snapshots, 60);
  equal(pde.spatial_points, 64);
  equal(pde.dx, 9.81747704246810349e-2);
  equal(pde.dt, 1.00000000000000002e-2);
  equal(pde.interior_points, 3596);
  equal(pde.residual_sum_squares, 1.48656280025936568e-14);
  equal(pde.law, "u_t = -0.049992*u +0.050018*u_xx");
  equal(pde.terms.length, 9);
  equal(pde.terms[1]!.label, "u");
  equal(pde.terms[1]!.u_power, 1);
  equal(pde.terms[1]!.derivative_order, 0);
  equal(pde.terms[1]!.coefficient, -4.99916540251027358e-2);
  equal(pde.terms[6]!.label, "u_xx");
  equal(pde.terms[6]!.derivative_order, 2);
  equal(pde.terms[6]!.coefficient, 5.00181836393979451e-2);

  // --- error paths: malformed discovery-engine input is rejected per shape ---
  throws(() => parseKoopmanReport(null), "null is not a koopman report");
  // The fixed method token must match exactly.
  throws(
    () => parseKoopmanReport({ ...JSON.parse(KOOPMAN), method: "dmd" }),
    "wrong koopman method token is rejected",
  );
  // A discrete eigenvalue missing its modulus is rejected.
  throws(
    () =>
      parseKoopmanReport({
        ...JSON.parse(KOOPMAN),
        discrete_eigenvalues: [{ re: 0.5, im: 0 }],
      }),
    "discrete eigenvalue missing modulus is rejected",
  );
  // rank must be a non-negative integer, not a float.
  throws(
    () => parseKoopmanReport({ ...JSON.parse(KOOPMAN), rank: 1.5 }),
    "fractional koopman rank is rejected",
  );
  throws(() => parseSdeReport(null), "null is not an sde report");
  throws(
    () => parseSdeReport({ ...JSON.parse(SDE), method: "kramers-moyal" }),
    "wrong sde method token is rejected",
  );
  // A bin count must be a non-negative integer.
  throws(
    () =>
      parseSdeReport({
        ...JSON.parse(SDE),
        states: [{ state: "x", trusted_bins: 1, drift: { terms: [], residual_sum_squares: 0 }, diffusion: { terms: [], residual_sum_squares: 0 }, bins: [{ x_center: 0, drift: 0, diffusion: 0, count: -1 }] }],
      }),
    "negative bin count is rejected",
  );
  // A state model missing its diffusion law is rejected.
  throws(
    () =>
      parseSdeReport({
        ...JSON.parse(SDE),
        states: [{ state: "x", trusted_bins: 1, drift: { terms: [], residual_sum_squares: 0 }, bins: [] }],
      }),
    "sde state missing diffusion is rejected",
  );
  throws(() => parsePdeReport(null), "null is not a pde report");
  throws(
    () => parsePdeReport({ ...JSON.parse(PDE), method: "pde" }),
    "wrong pde method token is rejected",
  );
  // A term's derivative_order must be a non-negative integer.
  throws(
    () =>
      parsePdeReport({
        ...JSON.parse(PDE),
        terms: [{ label: "u_x", u_power: 0, derivative_order: 1.5, coefficient: 1 }],
      }),
    "fractional derivative_order is rejected",
  );
  // Missing the terms array is rejected.
  throws(
    () => {
      const withoutTerms: Record<string, unknown> = { ...JSON.parse(PDE) };
      delete withoutTerms.terms;
      return parsePdeReport(withoutTerms);
    },
    "pde report missing terms is rejected",
  );

  // --- validate* returns issues instead of throwing ---
  const bad = validateStabilityReport({ world: 5 });
  equal(bad.ok, false);
  ok(!bad.ok && bad.issues.length > 0, "issues are reported");
}
