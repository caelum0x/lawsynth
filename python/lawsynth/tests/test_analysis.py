"""Tests for the CLI-backed engine analyses (``lawsynth.analysis``).

The module is a thin, dependency-free presentation layer over the compiled
``lawsynth`` CLI. Two tiers of tests reflect that:

* **Dep-free / offline** — import, error typing, JSON/text parsing from captured
  sample output, and the typed ``CliError`` / ``MissingBinaryError`` paths (via
  monkeypatching). These need no binary and run everywhere.
* **Live** — locate/build the CLI via ``benchmarks/_engine`` (``ensure_binary``,
  ``allow_build=True``) and actually invoke it on small deterministic
  worlds/datasets, asserting the parsed dataclasses. These **skip cleanly** with
  a clear message when the binary cannot be built (never a silent pass, never a
  hard failure on a binary-less environment).

The captured JSON fixtures below are verbatim CLI output, so the parse tests pin
the exact key contract the module depends on.
"""

from __future__ import annotations

import csv
import json
import math
import subprocess
import sys
from pathlib import Path

import pytest

import lawsynth
from lawsynth import analysis
from lawsynth.errors import LawSynthError, ValidationError


# --------------------------------------------------------------------------- #
# Live-binary plumbing: reuse benchmarks/_engine to locate/build the CLI       #
# --------------------------------------------------------------------------- #

_REPO_ROOT = analysis._repository_root()
_BENCHMARKS = _REPO_ROOT / "benchmarks" if _REPO_ROOT is not None else None


def _binary_or_skip() -> Path:
    """Locate (building once if needed) the CLI binary, else skip cleanly."""
    if _REPO_ROOT is None or _BENCHMARKS is None or not _BENCHMARKS.is_dir():
        pytest.skip("LawSynth repository / benchmarks not found; cannot locate the CLI")
    if str(_BENCHMARKS) not in sys.path:
        sys.path.insert(0, str(_BENCHMARKS))
    try:
        import _engine  # type: ignore[import-not-found]
    except Exception as error:  # pragma: no cover - environment dependent
        pytest.skip(f"cannot import benchmarks/_engine to locate the CLI: {error}")
    try:
        return _engine.ensure_binary(_REPO_ROOT, allow_build=True)
    except _engine.EngineUnavailable as error:
        pytest.skip(f"lawsynth CLI unavailable (build failed / offline): {error}")


def _write_csv(path: Path, header: list[str], rows: list[list[float]]) -> None:
    with path.open("w", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerow(header)
        writer.writerows(rows)


def _discover_stable_node(binary: Path, workdir: Path) -> Path:
    """Discover a stable-node world dx/dt=-x, dy/dt=-2y from its clean trajectory."""
    rows = [[i * 0.02, math.exp(-i * 0.02), math.exp(-2 * i * 0.02)] for i in range(400)]
    csv_path = workdir / "node.csv"
    world = workdir / "node.lsworld"
    _write_csv(csv_path, ["time", "x", "y"], rows)
    completed = subprocess.run(
        [str(binary), "discover", str(csv_path), "--time", "time",
         "--state", "x,y", "--output", str(world), "--degree", "1"],
        capture_output=True, text=True, check=False,
    )
    assert completed.returncode == 0, completed.stderr
    return world


def _new_world(binary: Path, workdir: Path, template: str) -> Path:
    """Instantiate a canonical parameterized world via ``lawsynth new TEMPLATE``.

    Discovered worlds inline their coefficients as constants (no free parameter),
    so bifurcation / sensitivity — which differentiate w.r.t. declared parameters —
    need a template world that actually declares them.
    """
    world = workdir / f"{template}.lsworld"
    completed = subprocess.run(
        [str(binary), "new", template, "--output", str(world)],
        capture_output=True, text=True, check=False,
    )
    assert completed.returncode == 0, completed.stderr
    return world


def _forced_dataset(workdir: Path) -> Path:
    """A forced linear system dx/dt=-x+u with u=cos(t), integrated by RK4."""
    def deriv(x: float, u: float) -> float:
        return -x + u

    x, t, dt = 1.0, 0.0, 0.02
    rows: list[list[float]] = []
    for _ in range(400):
        rows.append([t, x, math.cos(t)])
        k1 = deriv(x, math.cos(t))
        k2 = deriv(x + dt / 2 * k1, math.cos(t + dt / 2))
        k3 = deriv(x + dt / 2 * k2, math.cos(t + dt / 2))
        k4 = deriv(x + dt * k3, math.cos(t + dt))
        x = x + dt / 6 * (k1 + 2 * k2 + 2 * k3 + k4)
        t += dt
    csv_path = workdir / "forced.csv"
    _write_csv(csv_path, ["time", "x", "u"], rows)
    return csv_path


# --------------------------------------------------------------------------- #
# Captured CLI output — verbatim fixtures pinning the key contract             #
# --------------------------------------------------------------------------- #

_STABILITY_JSON = """{
  "world": "/tmp/node.lsworld",
  "states": ["x", "y"],
  "seeds_total": 25,
  "seeds_converged": 25,
  "fixed_points": [
    {
      "coordinates": [0.00000000000000000e0, 0.00000000000000000e0],
      "classification": "stable node",
      "inconclusive": false,
      "eigenvalues": [{"re": -2.00053337600112213e0, "im": 0.00000000000000000e0}, {"re": -1.00006666799976096e0, "im": 0.00000000000000000e0}]
    }
  ]
}
"""

_CONTROL_JSON = """{
  "source": "/tmp/forced.csv",
  "states": ["x"],
  "controls": ["u"],
  "equations": [
    {
      "state": "x",
      "residual_sum_squares": 1.88285338248170769e-5,
      "terms": [{"term": "u", "coefficient": 9.99914848930981481e-1}, {"term": "x", "coefficient": -9.99922037616008397e-1}]
    }
  ],
  "validation": {
    "in_sample": true,
    "per_state": [{"state": "x", "r_squared": 9.99999994388475089e-1, "rmse": 4.24469627949033190e-5}],
    "aggregate_r_squared": 9.99999994388475089e-1,
    "aggregate_rmse": 4.24469627949033190e-5
  }
}
"""

_CONTROL_JSON_NO_VALIDATION = """{
  "source": "/tmp/forced.csv",
  "states": ["x"],
  "controls": ["u"],
  "equations": [
    {"state": "x", "residual_sum_squares": 1.0e-5, "terms": [{"term": "x", "coefficient": -1.0e0}]}
  ],
  "validation": null
}
"""

_DOMAINS_LIST_TEXT = (
    "Domain presets (use with `domains show|run <name>`):\n\n"
    "  damped-oscillator\n"
    "    Damped linear harmonic oscillator: dx/dt = v, dv/dt = -x - 0.5 v.\n"
    "  lotka-volterra\n"
    "    Lotka-Volterra predator-prey: dprey/dt = 1.5 prey - prey predator.\n"
    "  brusselator\n"
    "    Brusselator autocatalytic kinetics: dx/dt = 1 - 4 x + x^2 y.\n"
)

_DOMAINS_SHOW_TEXT = (
    "Domain preset: lotka-volterra\n"
    "  Lotka-Volterra predator-prey.\n\n"
    "Reference law (state order: prey, predator):\n"
    "  d/dt prey = 1.5*prey + -1*prey*predator\n"
    "  d/dt predator = 0.75*prey*predator + -1*predator\n"
    "  initial: [10, 5]  dt=0.001  steps=4000  (4001 samples)\n\n"
    "Discovery configuration:\n"
    "  polynomial degree:  2\n"
    "  trigonometric:      false\n"
    "  rational:           false\n"
    "  template prior:     yes\n"
    "  unit hints:         (none)\n"
)

# Verbatim `--json` output captured from the built CLI (target/debug/lawsynth) on
# the worlds the live tests below build. These pin the exact key contract each
# parser depends on (bifurcation.rs / sensitivity.rs / estimate.rs / reduce.rs).

# `lawsynth new van-der-pol` then bifurcation --parameter mu --range -0.5:0.5
# --box -0.5:0.5,-0.5:0.5 --steps 21: a Hopf at mu* ~ 0 (complex pair crossing).
_BIFURCATION_JSON = """{
  "world": "/tmp/vdp.lsworld",
  "states": ["x", "y"],
  "parameter": "mu",
  "range": {"min": -5.00000000000000000e-1, "max": 5.00000000000000000e-1},
  "steps": 21,
  "branch_count": 1,
  "bifurcations": [
    {
      "parameter_value": -2.00000000003149048e-9,
      "kind": "hopf",
      "branch_id": 0,
      "fixed_point": [-6.25000051712731874e-11, 0.00000000000000000e0],
      "eigenvalue": {"re": -1.00000000001574524e-9, "im": 1.00000000000000000e0}
    }
  ]
}
"""

# `lawsynth new sir` then sensitivity --parameters beta --initial S=0.99
# --initial I=0.01 --dt 0.01 --steps 100. State order is I, R, S.
_SENSITIVITY_JSON = """{
  "world": "/tmp/sir.lsworld",
  "states": ["I", "R", "S"],
  "parameters": ["beta"],
  "final_time": 1.00000000000000000e0,
  "sensitivities": [
    {"state": "I", "parameter": "beta", "value": 1.60633892812004908e-2},
    {"state": "R", "parameter": "beta", "value": 6.88354736877626262e-4},
    {"state": "S", "parameter": "beta", "value": -1.67517440180781144e-2}
  ]
}
"""

# `lawsynth estimate vdp --box -0.5:0.5,-0.5:0.5 --measure x --poles -2,-3`:
# pole placement, error poles land at -2 and -3, gain L is 2x1, covariance null.
_ESTIMATE_POLE_JSON = """{
  "world": "/tmp/vdp.lsworld",
  "states": ["x", "y"],
  "fixed_point": [0.00000000000000000e0, 0.00000000000000000e0],
  "fixed_points_found": 1,
  "measured": ["x"],
  "method": "pole_placement",
  "gain": [[6.00000000000000000e0], [1.10000000000000000e1]],
  "error_poles": [{"re": -3.00000000000000044e0, "im": 0.00000000000000000e0}, {"re": -2.00000000000000000e0, "im": 0.00000000000000000e0}],
  "convergent": true,
  "covariance": null
}
"""

# `lawsynth estimate node --box -1:1,-1:1 --measure x,y --kalman` on the stable
# node diag(-1,-2): steady-state Kalman gain and covariance P (both 2x2).
_ESTIMATE_KALMAN_JSON = """{
  "world": "/tmp/node.lsworld",
  "states": ["x", "y"],
  "fixed_point": [0.00000000000000000e0, 0.00000000000000000e0],
  "fixed_points_found": 1,
  "measured": ["x", "y"],
  "method": "kalman",
  "gain": [[4.14194036553732658e-1, 0.00000000000000000e0], [0.00000000000000000e0, 2.36011680217161873e-1]],
  "error_poles": [{"re": -2.23654505621828381e0, "im": 0.00000000000000000e0}, {"re": -1.41426070455349362e0, "im": 0.00000000000000000e0}],
  "convergent": true,
  "covariance": [[4.14194036553732658e-1, 0.00000000000000000e0], [0.00000000000000000e0, 2.36011680217161873e-1]]
}
"""

# `lawsynth reduce node --box -1:1,-1:1 --order 1` on the stable node diag(-1,-2):
# Hankel singular values, reduced order 1, and the reduced A/B/C matrices.
_REDUCE_JSON = """{
  "world": "/tmp/node.lsworld",
  "states": ["x", "y"],
  "fixed_point": [0.00000000000000000e0, 0.00000000000000000e0],
  "measured": null,
  "hankel_singular_values": [4.99966668222282473e-1, 2.49933345775741528e-1],
  "order": 1,
  "error_bound": 4.99866691551483056e-1,
  "reduced": {
    "a": [[-1.00006666799976052e0]],
    "b": [[9.99999999999999889e-1, 0.00000000000000000e0]],
    "c": [[9.99999999999999889e-1], [0.00000000000000000e0]]
  }
}
"""


# --------------------------------------------------------------------------- #
# Import & error typing (no binary required)                                   #
# --------------------------------------------------------------------------- #


def test_public_symbols_import_lazily():
    assert callable(lawsynth.stability)
    assert callable(lawsynth.discover_controlled)
    assert callable(lawsynth.domains)
    assert callable(lawsynth.bifurcation)
    assert callable(lawsynth.sensitivity)
    assert callable(lawsynth.estimate)
    assert callable(lawsynth.reduce)
    assert lawsynth.StabilityReport is analysis.StabilityReport
    assert lawsynth.ControlledModel is analysis.ControlledModel
    assert lawsynth.BifurcationReport is analysis.BifurcationReport
    assert lawsynth.SensitivityReport is analysis.SensitivityReport
    assert lawsynth.EstimateReport is analysis.EstimateReport
    assert lawsynth.ReductionReport is analysis.ReductionReport
    assert lawsynth.CliError is analysis.CliError
    assert lawsynth.MissingBinaryError is analysis.MissingBinaryError


def test_error_hierarchy():
    assert issubclass(analysis.AnalysisError, LawSynthError)
    assert issubclass(analysis.MissingBinaryError, analysis.AnalysisError)
    assert issubclass(analysis.CliError, analysis.AnalysisError)
    assert issubclass(analysis.MissingBinaryError, LawSynthError)


def test_cli_error_carries_command_and_stderr():
    error = analysis.CliError(command=["lawsynth", "stability", "w"], returncode=2, stderr="boom\n")
    assert error.returncode == 2
    assert error.command == ("lawsynth", "stability", "w")
    assert error.stderr == "boom"
    assert "boom" in str(error)
    assert "stability" in str(error)


# --------------------------------------------------------------------------- #
# JSON / text parsing from captured output (no binary required)                #
# --------------------------------------------------------------------------- #


def test_parse_stability_dataclasses():
    report = analysis._parse_stability(json.loads(_STABILITY_JSON))
    assert isinstance(report, analysis.StabilityReport)
    assert report.states == ("x", "y")
    assert report.seeds_total == 25
    assert report.seeds_converged == 25
    assert len(report.fixed_points) == 1
    point = report.fixed_points[0]
    assert point.classification == "stable node"
    assert point.inconclusive is False
    assert point.coordinates == (0.0, 0.0)
    assert point.at(report.states) == {"x": 0.0, "y": 0.0}
    # A stable node has all-real, all-negative eigenvalues.
    assert all(eig.im == 0.0 and eig.re < 0.0 for eig in point.eigenvalues)


def test_parse_controlled_with_validation():
    model = analysis._parse_controlled(json.loads(_CONTROL_JSON))
    assert model.states == ("x",)
    assert model.controls == ("u",)
    assert len(model.equations) == 1
    equation = model.equations[0]
    assert equation.state == "x"
    coeffs = {term.term: term.coefficient for term in equation.terms}
    assert math.isclose(coeffs["x"], -1.0, abs_tol=1e-3)
    assert math.isclose(coeffs["u"], 1.0, abs_tol=1e-3)
    assert "u" in equation.expression() and "x" in equation.expression()
    assert model.validation is not None
    assert model.validation.in_sample is True
    assert math.isclose(model.validation.aggregate_r_squared, 1.0, abs_tol=1e-6)
    assert model.validation.per_state[0].state == "x"


def test_parse_controlled_without_validation():
    model = analysis._parse_controlled(json.loads(_CONTROL_JSON_NO_VALIDATION))
    assert model.validation is None
    assert model.equations[0].terms[0].term == "x"


def test_parse_domain_names():
    names = analysis._parse_domain_names(_DOMAINS_LIST_TEXT)
    assert names == ["damped-oscillator", "lotka-volterra", "brusselator"]


def test_parse_domain_show():
    show = analysis._parse_domain_show(_DOMAINS_SHOW_TEXT)
    assert show["name"] == "lotka-volterra"
    assert show["state_order"] == ["prey", "predator"]
    assert show["reference_laws"]["prey"] == "1.5*prey + -1*prey*predator"
    assert show["discovery"]["polynomial_degree"] == 2
    assert show["discovery"]["trigonometric"] is False
    assert "text" in show


def test_parse_bifurcation_dataclasses():
    report = analysis._parse_bifurcation(json.loads(_BIFURCATION_JSON))
    assert isinstance(report, analysis.BifurcationReport)
    assert report.states == ("x", "y")
    assert report.parameter == "mu"
    assert report.range_min == pytest.approx(-0.5)
    assert report.range_max == pytest.approx(0.5)
    assert report.steps == 21
    assert report.branch_count == 1
    assert len(report.bifurcations) == 1
    bif = report.bifurcations[0]
    # A Hopf: complex pair crossing the imaginary axis near mu* = 0.
    assert bif.kind == "hopf"
    assert bif.branch_id == 0
    assert abs(bif.parameter_value) < 1e-6
    assert bif.eigenvalue.im != 0.0
    assert abs(bif.eigenvalue.re) < 1e-6
    assert bif.at(report.states)["x"] == pytest.approx(bif.fixed_point[0])


def test_parse_sensitivity_dataclasses():
    report = analysis._parse_sensitivity(json.loads(_SENSITIVITY_JSON))
    assert isinstance(report, analysis.SensitivityReport)
    assert report.states == ("I", "R", "S")
    assert report.parameters == ("beta",)
    assert report.final_time == pytest.approx(1.0)
    # More infectiousness pushes S down and I up: signs are physically fixed.
    assert report.value("S", "beta") < 0.0
    assert report.value("I", "beta") > 0.0
    matrix = report.matrix()
    assert len(matrix) == 3 and all(len(row) == 1 for row in matrix)
    with pytest.raises(KeyError):
        report.value("S", "gamma")


def test_parse_estimate_pole_placement():
    report = analysis._parse_estimate(json.loads(_ESTIMATE_POLE_JSON))
    assert isinstance(report, analysis.EstimateReport)
    assert report.method == "pole_placement"
    assert report.measured == ("x",)
    assert report.fixed_points_found == 1
    # L is states x outputs = 2 x 1.
    assert len(report.gain) == 2 and all(len(row) == 1 for row in report.gain)
    poles = sorted(pole.re for pole in report.error_poles)
    assert poles == pytest.approx([-3.0, -2.0])
    assert all(pole.im == 0.0 for pole in report.error_poles)
    assert report.convergent is True
    assert report.covariance is None


def test_parse_estimate_kalman_has_covariance():
    report = analysis._parse_estimate(json.loads(_ESTIMATE_KALMAN_JSON))
    assert report.method == "kalman"
    assert report.measured == ("x", "y")
    assert report.covariance is not None
    assert len(report.covariance) == 2 and len(report.covariance[0]) == 2
    # Stable error dynamics: every eigenvalue has negative real part.
    assert all(pole.re < 0.0 for pole in report.error_poles)
    assert report.convergent is True


def test_parse_reduce_dataclasses():
    report = analysis._parse_reduce(json.loads(_REDUCE_JSON))
    assert isinstance(report, analysis.ReductionReport)
    assert report.states == ("x", "y")
    assert report.measured is None
    assert report.order == 1
    # Hankel singular values are non-increasing and positive.
    sigmas = report.hankel_singular_values
    assert len(sigmas) == 2 and sigmas[0] > sigmas[1] > 0.0
    assert report.error_bound > 0.0
    # Reduced A is 1x1, B is 1x2, C is 2x1 (order 1 from a 2-state system).
    assert len(report.reduced.a) == 1 and len(report.reduced.a[0]) == 1
    assert len(report.reduced.b) == 1 and len(report.reduced.b[0]) == 2
    assert len(report.reduced.c) == 2 and len(report.reduced.c[0]) == 1


def test_parse_new_reports_reject_malformed_input():
    # Missing required keys / wrong container types must raise AnalysisError, not
    # silently produce a half-filled dataclass.
    with pytest.raises(analysis.AnalysisError):
        analysis._parse_bifurcation({"states": ["x"], "parameter": "mu"})  # no 'range'
    with pytest.raises(analysis.AnalysisError):
        analysis._parse_bifurcation(
            {"states": ["x"], "parameter": "mu", "range": {"min": 0.0, "max": 1.0},
             "steps": 3, "branch_count": 0, "bifurcations": {"not": "a list"}}
        )
    with pytest.raises(analysis.AnalysisError):
        analysis._parse_sensitivity({"states": ["x"], "parameters": ["a"]})  # no sensitivities
    with pytest.raises(analysis.AnalysisError):
        analysis._parse_estimate(
            {"states": ["x"], "fixed_point": [0.0], "fixed_points_found": 1,
             "measured": ["x"], "method": "pole_placement", "gain": [[1.0]],
             "error_poles": "nope", "convergent": True, "covariance": None}
        )
    with pytest.raises(analysis.AnalysisError):
        analysis._parse_reduce(
            {"states": ["x"], "fixed_point": [0.0], "measured": None,
             "hankel_singular_values": [1.0], "order": 1, "error_bound": 0.1,
             "reduced": {"a": [[1.0]], "b": [[1.0]]}}  # missing 'c'
        )


def test_new_report_arg_validation():
    # Client-side validation happens before any subprocess is spawned.
    with pytest.raises(ValidationError):
        lawsynth.bifurcation("w.lsworld", parameter="  ", range=(0.0, 1.0), box=[(-1.0, 1.0)])
    with pytest.raises(ValidationError):
        lawsynth.bifurcation("w.lsworld", parameter="mu", range=(1.0, 0.0), box=[(-1.0, 1.0)])
    with pytest.raises(ValidationError):
        lawsynth.sensitivity("w.lsworld", parameters=[])
    with pytest.raises(ValidationError):
        lawsynth.estimate("w.lsworld", box=[(-1.0, 1.0)], measure=["x"])  # no poles, no kalman
    with pytest.raises(ValidationError):
        lawsynth.estimate("w.lsworld", box=[(-1.0, 1.0)], measure=["x"], poles=[-1.0], kalman=True)
    with pytest.raises(ValidationError):
        lawsynth.reduce("w.lsworld", box=[(-1.0, 1.0)])  # neither order nor tolerance
    with pytest.raises(ValidationError):
        lawsynth.reduce("w.lsworld", box=[(-1.0, 1.0)], order=1, tolerance=0.1)


def test_format_helpers_for_new_commands():
    assert analysis._format_range("0:1") == "0:1"
    assert analysis._format_range((-0.5, 0.5)) == "-0.5:0.5"
    with pytest.raises(ValidationError):
        analysis._format_range("   ")
    assert analysis._format_poles([-2.0, -3.0]) == "-2.0,-3.0"
    assert analysis._format_poles([(-1.0, 2.0)]) == "-1.0:2.0"
    assert analysis._format_poles(["-1:2"]) == "-1:2"
    assert analysis._format_identifiers(["a", "b"], label="x") == "a,b"
    with pytest.raises(ValidationError):
        analysis._format_identifiers([], label="x")
    assert analysis._initial_args({"S": 0.99}) == ["--initial", "S=0.99"]
    assert analysis._initial_args([("I", 0.01)]) == ["--initial", "I=0.01"]
    assert analysis._initial_args(None) == []


def test_run_cli_rejects_non_object_json(monkeypatch):
    monkeypatch.setattr(analysis, "_invoke", lambda *a, **k: "[1, 2, 3]")
    with pytest.raises(analysis.AnalysisError):
        analysis._run_cli(["domains", "run", "x", "--json"])


# --------------------------------------------------------------------------- #
# Typed error paths via monkeypatching (no binary required)                    #
# --------------------------------------------------------------------------- #


class _FakeCompleted:
    def __init__(self, returncode: int, stdout: str = "", stderr: str = "") -> None:
        self.returncode = returncode
        self.stdout = stdout
        self.stderr = stderr


def test_cli_error_raised_on_nonzero_exit(monkeypatch):
    monkeypatch.setattr(
        analysis.subprocess, "run",
        lambda *a, **k: _FakeCompleted(3, stdout="", stderr="world not found"),
    )
    with pytest.raises(analysis.CliError) as info:
        analysis._invoke(["stability", "missing.lsworld"], binary=Path("/bin/true"))
    assert info.value.returncode == 3
    assert "world not found" in info.value.stderr
    assert "stability" in " ".join(info.value.command)


def test_missing_binary_error_when_no_candidates(monkeypatch):
    monkeypatch.setattr(analysis, "_candidate_binaries", lambda: [Path("/nonexistent/lawsynth")])
    with pytest.raises(analysis.MissingBinaryError) as info:
        analysis._locate_binary()
    assert "cargo build" in str(info.value)
    assert "LAWSYNTH_BIN" in str(info.value)


def test_format_box_accepts_pairs_and_string():
    assert analysis._format_box("-1:1, -2:2") == "-1:1, -2:2"
    assert analysis._format_box([(-1.0, 1.0), (-2.0, 2.0)]) == "-1.0:1.0,-2.0:2.0"
    with pytest.raises(ValidationError):
        analysis._format_box([])
    with pytest.raises(ValidationError):
        analysis._format_box([(1.0, -1.0)])


def test_discover_controlled_validates_inputs():
    with pytest.raises(ValidationError):
        lawsynth.discover_controlled("x.csv", states=[], controls=["u"])
    with pytest.raises(ValidationError):
        lawsynth.discover_controlled("x.csv", states=["x"], controls=[])


# --------------------------------------------------------------------------- #
# Live tests — real CLI invocation (skip cleanly when the binary is absent)    #
# --------------------------------------------------------------------------- #


def test_domains_lists_known_presets():
    _binary_or_skip()
    names = lawsynth.domains()
    for expected in ("damped-oscillator", "lotka-volterra", "brusselator"):
        assert expected in names


def test_domain_run_recovers_preset():
    _binary_or_skip()
    result = lawsynth.domain_run("damped-oscillator")
    assert result["preset"] == "damped-oscillator"
    assert result["recovered"] is True
    assert isinstance(result["laws"], list) and result["laws"]
    assert all("rhs_rmse" in entry for entry in result["recovery"])


def test_domain_show_returns_reference_and_config():
    _binary_or_skip()
    show = lawsynth.domain_show("lotka-volterra")
    assert show["name"] == "lotka-volterra"
    assert "prey" in show["reference_laws"]
    assert show["discovery"]["polynomial_degree"] == 2


def test_stability_classifies_stable_node(tmp_path):
    binary = _binary_or_skip()
    world = _discover_stable_node(binary, tmp_path)
    report = lawsynth.stability(world, box=[(-1.0, 1.0), (-1.0, 1.0)])
    assert report.states == ("x", "y")
    assert report.seeds_converged > 0
    assert len(report.fixed_points) == 1
    point = report.fixed_points[0]
    assert point.classification == "stable node"
    # Origin is the fixed point; both eigenvalues real and negative.
    assert all(abs(value) < 1e-6 for value in point.coordinates)
    assert all(eig.im == 0.0 and eig.re < 0.0 for eig in point.eigenvalues)


def test_discover_controlled_forced_system_validates(tmp_path):
    _binary_or_skip()
    dataset = _forced_dataset(tmp_path)
    model = lawsynth.discover_controlled(
        dataset, states=["x"], controls=["u"], degree=1, validate=True
    )
    assert model.states == ("x",)
    assert model.controls == ("u",)
    coeffs = {term.term: term.coefficient for term in model.equations[0].terms}
    assert math.isclose(coeffs["x"], -1.0, abs_tol=1e-2)
    assert math.isclose(coeffs["u"], 1.0, abs_tol=1e-2)
    assert model.validation is not None
    # Clean forced linear system, in-sample rollout -> R2 near 1.
    assert model.validation.aggregate_r_squared > 0.99


def test_stability_convenience_method_on_result(tmp_path):
    binary = _binary_or_skip()
    # Drive a real DiscoveryResult through the CLI stability path via .stability().
    try:
        import lawsynth._native  # noqa: F401
    except ModuleNotFoundError as error:
        if error.name == "lawsynth._native":
            pytest.skip("native extension not built; DiscoveryResult.stability needs a live world")
        raise
    times = [i * 0.02 for i in range(400)]
    columns = {
        "x": [math.exp(-t) for t in times],
        "y": [math.exp(-2 * t) for t in times],
    }
    study = lawsynth.Study.from_columns(times, columns, state=["x", "y"], name="node")
    result = study.discover(polynomial_degree=1)
    assert hasattr(result, "stability")
    report = result.stability(box=[(-1.0, 1.0), (-1.0, 1.0)])
    assert report.fixed_points[0].classification == "stable node"


def test_cli_error_on_unknown_preset():
    _binary_or_skip()
    with pytest.raises(analysis.CliError):
        lawsynth.domain_run("navier-stokes-not-a-preset")


def test_bifurcation_detects_hopf_in_van_der_pol(tmp_path):
    binary = _binary_or_skip()
    world = _new_world(binary, tmp_path, "van-der-pol")
    # Van der Pol dx/dt=y, dy/dt=mu(1-x^2)y - x: at the origin the Jacobian is
    # [[0,1],[-1,mu]], whose complex-pair eigenvalues cross i-axis at mu*=0 (Hopf).
    report = lawsynth.bifurcation(
        world, parameter="mu", range=(-0.5, 0.5), box=[(-0.5, 0.5), (-0.5, 0.5)], steps=21
    )
    assert report.parameter == "mu"
    assert report.states == ("x", "y")
    assert report.branch_count >= 1
    assert any(bif.kind == "hopf" for bif in report.bifurcations)
    hopf = next(bif for bif in report.bifurcations if bif.kind == "hopf")
    assert abs(hopf.parameter_value) < 1e-3
    assert hopf.eigenvalue.im != 0.0


def test_bifurcation_rejects_unparameterized_world(tmp_path):
    binary = _binary_or_skip()
    # A discovered world inlines coefficients as constants: no free parameter.
    world = _discover_stable_node(binary, tmp_path)
    with pytest.raises(analysis.CliError):
        lawsynth.bifurcation(
            world, parameter="mu", range=(-1.0, 1.0), box=[(-1.0, 1.0), (-1.0, 1.0)]
        )


def test_sensitivity_signs_in_sir(tmp_path):
    binary = _binary_or_skip()
    world = _new_world(binary, tmp_path, "sir")
    # dS/dt=-beta*S*I: a larger transmission rate drives S down and I up, so the
    # signs of the final-time sensitivities are physically determined.
    report = lawsynth.sensitivity(
        world, parameters=["beta"], initial={"S": 0.99, "I": 0.01}, dt=0.01, steps=100
    )
    assert "beta" in report.parameters
    assert report.final_time == pytest.approx(1.0)
    assert report.value("S", "beta") < 0.0
    assert report.value("I", "beta") > 0.0


def test_estimate_pole_placement_on_van_der_pol(tmp_path):
    binary = _binary_or_skip()
    # Van der Pol's origin has the coupled Jacobian [[0,1],[-1,mu]], which is
    # observable from x — so single-output pole placement is well posed there.
    # (A diagonal, decoupled node would be unobservable from a single state.)
    world = _new_world(binary, tmp_path, "van-der-pol")
    report = lawsynth.estimate(
        world, box=[(-0.5, 0.5), (-0.5, 0.5)], measure=["x"], poles=[-2.0, -3.0]
    )
    assert report.method == "pole_placement"
    assert report.measured == ("x",)
    # L is states x outputs = 2 x 1.
    assert len(report.gain) == 2 and all(len(row) == 1 for row in report.gain)
    placed = sorted(pole.re for pole in report.error_poles)
    assert placed == pytest.approx([-3.0, -2.0], abs=1e-6)
    # Error poles at -2, -3 have negative real part: the observer converges.
    assert report.convergent is True
    assert report.covariance is None


def test_estimate_kalman_on_stable_node(tmp_path):
    binary = _binary_or_skip()
    world = _discover_stable_node(binary, tmp_path)
    # Full measurement (x and y) keeps the system observable for the Kalman design.
    report = lawsynth.estimate(
        world, box=[(-1.0, 1.0), (-1.0, 1.0)], measure=["x", "y"], kalman=True
    )
    assert report.method == "kalman"
    assert report.covariance is not None
    assert len(report.covariance) == 2 and len(report.covariance[0]) == 2
    assert all(pole.re < 0.0 for pole in report.error_poles)
    assert report.convergent is True


def test_reduce_stable_node_to_order_one(tmp_path):
    binary = _binary_or_skip()
    world = _discover_stable_node(binary, tmp_path)
    # A = diag(-1,-2) is Hurwitz, so balanced truncation is well posed.
    report = lawsynth.reduce(world, box=[(-1.0, 1.0), (-1.0, 1.0)], order=1)
    assert report.order == 1
    assert report.measured is None
    sigmas = report.hankel_singular_values
    assert len(sigmas) == 2 and sigmas[0] >= sigmas[1] > 0.0
    assert report.error_bound > 0.0
    assert len(report.reduced.a) == 1 and len(report.reduced.a[0]) == 1


def test_reduce_rejects_unstable_fixed_point(tmp_path):
    binary = _binary_or_skip()
    # Van der Pol's origin (mu=1) is an unstable focus: not Hurwitz, so balanced
    # truncation is rejected by the engine with an honest error.
    world = _new_world(binary, tmp_path, "van-der-pol")
    with pytest.raises(analysis.CliError):
        lawsynth.reduce(world, box=[(-0.5, 0.5), (-0.5, 0.5)], order=1)


def test_new_analysis_convenience_methods_on_result(tmp_path):
    _binary_or_skip()
    try:
        import lawsynth._native  # noqa: F401
    except ModuleNotFoundError as error:
        if error.name == "lawsynth._native":
            pytest.skip("native extension not built; convenience methods need a live world")
        raise
    times = [i * 0.02 for i in range(400)]
    columns = {
        "x": [math.exp(-t) for t in times],
        "y": [math.exp(-2 * t) for t in times],
    }
    study = lawsynth.Study.from_columns(times, columns, state=["x", "y"], name="node")
    result = study.discover(polynomial_degree=1)
    # The discovered (Hurwitz) node supports reduce/estimate via the attached
    # convenience methods, mirroring .stability(). The node is diagonal, so the
    # Kalman observer uses full (x, y) measurement to stay observable.
    assert hasattr(result, "reduce") and hasattr(result, "estimate")
    reduction = result.reduce(box=[(-1.0, 1.0), (-1.0, 1.0)], order=1)
    assert reduction.order == 1
    observer = result.estimate(box=[(-1.0, 1.0), (-1.0, 1.0)], measure=["x", "y"], kalman=True)
    assert observer.convergent is True
