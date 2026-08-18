import {
  analysisReportLabel,
  bifurcationView,
  classificationBadge,
  controlView,
  domainPresetList,
  domainRunView,
  estimateView,
  formatCoordinates,
  formatEigenvalue,
  formatScalar,
  isAnalysisReport,
  reductionView,
  sensitivityView,
  stabilityView,
} from "../src/analysis.js";
import { parseRoute, routePath } from "../src/routes.js";
import { deepEqual, equal, rejects } from "./support.js";

// All report fixtures below are VERBATIM `lawsynth ... --json` output, copied
// from packages/world-schema/tests/analysis.test.ts (which captured them from the
// real engine debug binary). The two empty fixtures are the same shape with an
// empty result set — the honest engine output when nothing is found in the box.

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

// Same command against a box with no equilibrium: zero fixed points located.
const STABILITY_EMPTY = `{
  "world": "linear-drift.lsworld",
  "states": ["x", "y"],
  "seeds_total": 25,
  "seeds_converged": 4,
  "fixed_points": []
}`;

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

// Same continuation over a range with no stability change: no bifurcation found.
const BIFURCATION_EMPTY = `{
  "world": "decay2d.lsworld",
  "states": ["x", "y"],
  "parameter": "k",
  "range": {"min": 1.00000000000000000e0, "max": 2.00000000000000000e0},
  "steps": 11,
  "branch_count": 1,
  "bifurcations": []
}`;

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

export async function analysisTests(): Promise<void> {
  // --- presentation helpers ---
  equal(formatScalar(0), "0");
  equal(formatScalar(1), "1");
  equal(formatScalar(-2.04166666104394379), "-2.04167");
  equal(formatCoordinates([0, 0]), "(0, 0)");
  equal(formatEigenvalue({ re: -2.04166666104394379, im: 0 }), "-2.04167");
  equal(formatEigenvalue({ re: 0, im: 9.99984274058213329e-1 }), "0 + 0.999984 i");
  equal(formatEigenvalue({ re: -5.64089737615183995e-1, im: -2.18790932903600099 }), "-0.56409 - 2.18791 i");
  equal(classificationBadge("saddle").tone, "error");
  equal(classificationBadge("stable spiral").stability, "stable");
  equal(classificationBadge("marginal (inconclusive)").stability, "inconclusive");

  // --- stability: stable node → definitive stable verdict ---
  const stable = stabilityView(JSON.parse(STABILITY_STABLE_NODE));
  equal(stable.world, "decay2d.lsworld");
  equal(stable.empty, false);
  equal(stable.rows.length, 1);
  const node = stable.rows[0]!;
  equal(node.classification, "stable node");
  equal(node.label, "Stable node");
  equal(node.tone, "success");
  equal(node.stability, "stable");
  equal(node.inconclusive, false);
  equal(node.coordinatesDisplay, "(0, 0)");
  equal(node.eigenvalues[0]!.display, "-2.04167");
  equal(node.eigenvalues[0]!.complex, false);
  equal(stable.stableCount, 1);
  equal(stable.convergenceRatio, 1);
  equal(stable.summary, "1 fixed point — 1 stable, 0 unstable, 0 inconclusive (25/25 seeds converged).");

  // --- stability: center → shown as inconclusive, NOT a stable/unstable verdict ---
  const center = stabilityView(JSON.parse(STABILITY_CENTER));
  const centerRow = center.rows[0]!;
  equal(centerRow.inconclusive, true);
  equal(centerRow.stability, "inconclusive");
  equal(centerRow.tone, "warning");
  equal(centerRow.label, "Center (marginal, inconclusive)");
  equal(centerRow.eigenvalues[1]!.complex, true);
  equal(centerRow.eigenvalues[1]!.display, "0 + 0.999984 i");
  equal(center.inconclusiveCount, 1);
  equal(center.stableCount, 0);
  equal(center.unstableCount, 0);

  // --- stability: empty box → honest "none found" message ---
  const empty = stabilityView(JSON.parse(STABILITY_EMPTY));
  equal(empty.empty, true);
  equal(empty.rows.length, 0);
  equal(empty.summary, "No fixed points found in the searched region.");
  equal(empty.convergenceRatio, 4 / 25);

  // --- control: with in-sample validation ---
  const control = controlView(JSON.parse(CONTROL_VALIDATED));
  equal(control.source, "forced1d.csv");
  equal(control.validated, true);
  equal(control.validation !== null, true);
  equal(control.validation!.inSample, true);
  equal(control.validation!.perState[0]!.state, "x");
  equal(control.equations[0]!.expression, "d/dt x = 0.999394 * u + -0.495801 * x");
  equal(control.validationStatus.startsWith("In-sample validated"), true);

  // --- control: null validation → shown as "Not validated", never a fabricated pass ---
  const bare = controlView(JSON.parse(CONTROL_UNVALIDATED));
  equal(bare.validated, false);
  equal(bare.validation, null);
  equal(bare.validationStatus, "Not validated");

  // --- domains: recovery table + preset list ---
  const domain = domainRunView(JSON.parse(DOMAIN_RUN));
  equal(domain.preset, "damped-oscillator");
  equal(domain.recovered, true);
  equal(domain.recoveredLabel, "Recovered");
  equal(domain.tone, "success");
  equal(domain.laws.length, 2);
  equal(domain.recovery.length, 2);
  equal(domain.recovery[1]!.state, "v");
  equal(domain.recovery[1]!.termsMatch, true);
  equal(domain.recovery[1]!.withinTolerance, true);
  const presets = domainPresetList([domain]);
  equal(presets.length, 1);
  equal(presets[0]!.preset, "damped-oscillator");
  equal(presets[0]!.recovered, true);

  // --- bifurcation: a real Hopf, and the empty-range honest case ---
  const bifurcation = bifurcationView(JSON.parse(BIFURCATION_HOPF));
  equal(bifurcation.parameter, "mu");
  equal(bifurcation.rangeDisplay, "[-1, 1]");
  equal(bifurcation.rows.length, 1);
  equal(bifurcation.rows[0]!.kind, "hopf");
  equal(bifurcation.rows[0]!.kindLabel, "Hopf");
  equal(bifurcation.rows[0]!.eigenvalue.complex, true);
  equal(bifurcation.rows[0]!.fixedPointDisplay, "(0, 0)");
  equal(bifurcation.summary, "1 bifurcation across 1 branch.");
  const noBifurcation = bifurcationView(JSON.parse(BIFURCATION_EMPTY));
  equal(noBifurcation.empty, true);
  equal(noBifurcation.summary, "No bifurcations detected across the swept range.");

  // --- sensitivity: state × parameter grid + peak ---
  const sensitivity = sensitivityView(JSON.parse(SENSITIVITY));
  equal(sensitivity.finalTime, 0.5);
  equal(sensitivity.rows.length, 2);
  equal(sensitivity.rows[0]!.state, "x");
  equal(sensitivity.rows[0]!.cells.length, 2);
  equal(sensitivity.rows[0]!.cells[0]!.parameter, "alpha");
  equal(sensitivity.rows[0]!.cells[0]!.display, "0.718024");
  deepEqual(sensitivity.peak, { state: "x", parameter: "alpha", value: 7.18024197761129912e-1 });

  // --- estimate: pole placement → covariance null ---
  const poles = estimateView(JSON.parse(ESTIMATE_POLES));
  equal(poles.method, "pole_placement");
  equal(poles.methodLabel, "Pole placement (Ackermann)");
  equal(poles.hasCovariance, false);
  equal(poles.covariance, null);
  equal(poles.convergent, true);
  equal(poles.convergentLabel, "Convergent");
  equal(poles.convergentTone, "success");
  equal(poles.errorPoles.length, 2);

  // --- estimate: Kalman → non-null covariance ---
  const kalman = estimateView(JSON.parse(ESTIMATE_KALMAN));
  equal(kalman.method, "kalman");
  equal(kalman.methodLabel, "Kalman filter (steady-state)");
  equal(kalman.hasCovariance, true);
  equal(kalman.covariance !== null, true);
  equal(kalman.covariance![0]![0], 4.261833187676622);

  // --- reduce: C = I (measured null) → honest "all states" label ---
  const reduction = reductionView(JSON.parse(REDUCTION));
  equal(reduction.measured, null);
  equal(reduction.measuredLabel, "C = I (all states measured)");
  equal(reduction.order, 1);
  equal(reduction.retained, 1);
  equal(reduction.discarded, 1);
  equal(reduction.reduced.a[0]![0], -1.2033972395446968e-1);

  // --- reduce: --measure selects C, so measured is a state list ---
  const reductionMeasured = reductionView(JSON.parse(REDUCTION_MEASURED));
  equal(reductionMeasured.measured !== null, true);
  equal(reductionMeasured.measuredLabel, "theta");
  equal(reductionMeasured.order, 2);

  // --- parsers reject malformed engine JSON (surfaced as thrown SchemaValidationError) ---
  await rejects(() => Promise.resolve(stabilityView({ world: "w" })), /fixed_points/);
  await rejects(
    () => Promise.resolve(controlView({ source: "s", states: [], controls: [], equations: [] })),
    /validation/,
  );

  // --- analysis routing is wired the same way as the other Studio routes ---
  equal(isAnalysisReport("stability"), true);
  equal(isAnalysisReport("nope"), false);
  equal(analysisReportLabel("reduce"), "Model reduction");
  const listRoute = parseRoute("/projects/project_1/analysis");
  deepEqual(listRoute, { name: "analysis", projectId: "project_1" });
  equal(routePath(listRoute), "/projects/project_1/analysis");
  const reportRoute = parseRoute("/projects/project_1/analysis/stability");
  deepEqual(reportRoute, { name: "analysis", projectId: "project_1", report: "stability" });
  equal(routePath(reportRoute), "/projects/project_1/analysis/stability");
  await rejects(() => Promise.resolve(parseRoute("/projects/project_1/analysis/bogus")), /analysis report/);
}
