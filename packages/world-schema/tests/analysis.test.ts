import {
  parseControlledModel,
  parseDomainRun,
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

  // --- validate* returns issues instead of throwing ---
  const bad = validateStabilityReport({ world: 5 });
  equal(bad.ok, false);
  ok(!bad.ok && bad.issues.length > 0, "issues are reported");
}
