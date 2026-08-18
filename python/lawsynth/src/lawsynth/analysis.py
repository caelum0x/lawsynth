"""A thin, typed Python client over the ``lawsynth`` CLI's engine analyses.

The Rust ``lawsynth`` binary is the single source of truth for the numerics. This
module does **not** re-implement any of them in Python — it shells out to the
compiled CLI (``lawsynth stability``, ``lawsynth control``, ``lawsynth domains``),
captures the CLI's stable ``--json`` (or, for the two ``domains`` subcommands that
have no ``--json``, its stable text) output, and parses it into small immutable
dataclasses. Parsing and presenting is all this layer does; the engine computes.
Keeping one source of truth is deliberate: a second Python implementation of the
fixed-point solver / SINDYc fit / round-trip would inevitably drift from the
engine and become filler.

**Determinism.** The engine already guarantees a deterministic result for a given
binary and inputs. The subprocess is invoked by the binary's *absolute path* with
a fixed, locale-pinned environment (``LC_ALL=C``, ``TZ=UTC``) so nothing depends
on the caller's locale, timezone, or wall clock. Given the same binary and the
same inputs, the parsed dataclasses are byte-for-byte reproducible.

**Graceful absence.** Like :mod:`lawsynth.export` and :mod:`lawsynth.tracking`,
the heavy dependency here (the built CLI binary) is required lazily and its
absence is reported honestly:

* when the binary cannot be located or built, a typed :class:`MissingBinaryError`
  (a subclass of the package base :class:`~lawsynth.errors.LawSynthError`) is
  raised with an actionable message;
* when the CLI exits non-zero, a typed :class:`CliError` carrying the exact
  command and the CLI's ``stderr`` is raised — the failure is never swallowed.

Binary discovery mirrors ``benchmarks/_engine.py`` exactly (the ``LAWSYNTH_BIN``
override, then ``target/debug/lawsynth``, then ``target/release/lawsynth`` under
the repository root), so a caller who has already built the CLI needs no extra
configuration. The SDK is a presentation layer: it requires the built CLI and
does nothing the engine cannot.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
from dataclasses import dataclass
from os import PathLike
from pathlib import Path
from typing import Mapping, Sequence

from .errors import LawSynthError, ValidationError

__all__ = [
    "AnalysisError",
    "CliError",
    "MissingBinaryError",
    "Eigenvalue",
    "FixedPoint",
    "StabilityReport",
    "ControlTerm",
    "ControlEquation",
    "StateScore",
    "ControlValidation",
    "ControlledModel",
    "stability",
    "discover_controlled",
    "domains",
    "domain_show",
    "domain_run",
]


# --------------------------------------------------------------------------- #
# Errors — mirror the graceful-absence contract of export.py / tracking.py     #
# --------------------------------------------------------------------------- #


class AnalysisError(LawSynthError):
    """Base class for failures of the CLI-backed engine analyses."""


class MissingBinaryError(AnalysisError):
    """Raised when the compiled ``lawsynth`` CLI cannot be located or built.

    The SDK's analysis client is a presentation layer over the engine; when the
    engine binary is absent there is nothing to present. The message names the
    concrete fix (build the CLI or set ``LAWSYNTH_BIN``), keeping the boundary
    honest rather than fabricating a result.
    """


class CliError(AnalysisError):
    """Raised when the ``lawsynth`` CLI exits non-zero.

    Carries the exact ``command`` that was run and the CLI's ``stderr`` so the
    failure is fully diagnosable and never silently swallowed.
    """

    def __init__(self, command: Sequence[str], returncode: int, stderr: str) -> None:
        self.command = tuple(str(part) for part in command)
        self.returncode = returncode
        self.stderr = stderr.strip()
        detail = self.stderr or "(no stderr)"
        super().__init__(
            f"lawsynth CLI exited with status {returncode} for command "
            f"{' '.join(self.command)!r}: {detail}"
        )


# --------------------------------------------------------------------------- #
# Typed results — deterministic images of the CLI's --json shapes              #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class Eigenvalue:
    """A complex Jacobian eigenvalue ``re + im i`` at a fixed point."""

    re: float
    im: float


@dataclass(frozen=True, slots=True)
class FixedPoint:
    """One located fixed point ``f(x)=0`` with its linear-stability verdict.

    ``coordinates`` are ordered like the report's ``states``; ``classification``
    is the engine's human label (``"stable node"``, ``"saddle"``, …) and
    ``inconclusive`` is set for non-hyperbolic (center/marginal) points the
    linearization cannot decide.
    """

    coordinates: tuple[float, ...]
    classification: str
    inconclusive: bool
    eigenvalues: tuple[Eigenvalue, ...]

    def at(self, states: Sequence[str]) -> dict[str, float]:
        """Map coordinates onto ``states`` (``{state: value}``)."""
        return {str(name): value for name, value in zip(states, self.coordinates)}


@dataclass(frozen=True, slots=True)
class StabilityReport:
    """The parsed result of ``lawsynth stability WORLD --box ... --json``.

    ``seeds_total`` / ``seeds_converged`` report the deterministic Newton search:
    an empty ``fixed_points`` means the search found nothing inside the box, not
    that the system has none.
    """

    world: str
    states: tuple[str, ...]
    seeds_total: int
    seeds_converged: int
    fixed_points: tuple[FixedPoint, ...]


@dataclass(frozen=True, slots=True)
class ControlTerm:
    """A single active library term ``coefficient * term`` in a fitted equation."""

    term: str
    coefficient: float


@dataclass(frozen=True, slots=True)
class ControlEquation:
    """One fitted state equation ``d/dt state = Σ coefficient·term``."""

    state: str
    residual_sum_squares: float
    terms: tuple[ControlTerm, ...]

    def expression(self) -> str:
        """Render the right-hand side as ``c0*t0 + c1*t1 + ...`` (``"0"`` if empty)."""
        if not self.terms:
            return "0"
        return " + ".join(f"{term.coefficient}*{term.term}" for term in self.terms)


@dataclass(frozen=True, slots=True)
class StateScore:
    """Per-state rollout score ``R²`` / ``RMSE`` from ``--validate``."""

    state: str
    r_squared: float
    rmse: float


@dataclass(frozen=True, slots=True)
class ControlValidation:
    """In-sample validation of a controlled model (open-loop rollout vs. observed).

    ``in_sample`` is always ``True`` — the CLI validates on the same data it fit,
    and labels it as such; open-loop rollout error grows with horizon.
    """

    in_sample: bool
    per_state: tuple[StateScore, ...]
    aggregate_r_squared: float
    aggregate_rmse: float


@dataclass(frozen=True, slots=True)
class ControlledModel:
    """The parsed result of ``lawsynth control ... --json`` (SINDYc).

    One :class:`ControlEquation` per state over the augmented library ``Θ(x, u)``;
    controls appear only inside library terms and are never predicted.
    ``validation`` is populated only when the command ran with ``validate=True``.
    """

    source: str
    states: tuple[str, ...]
    controls: tuple[str, ...]
    equations: tuple[ControlEquation, ...]
    validation: ControlValidation | None


# --------------------------------------------------------------------------- #
# Parsers — the JSON/text shapes each CLI subcommand emits                     #
#                                                                              #
# These operate on already-decoded data (dict / str) so parsing is fully       #
# testable with no binary present. Keys mirror crates/lawsynth-cli/src/*.rs.    #
# --------------------------------------------------------------------------- #


def _as_float(value: object, field: str) -> float:
    try:
        return float(value)  # type: ignore[arg-type]
    except (TypeError, ValueError) as error:
        raise AnalysisError(f"CLI JSON field {field!r} is not a number: {value!r}") from error


def _as_int(value: object, field: str) -> int:
    try:
        return int(value)  # type: ignore[arg-type]
    except (TypeError, ValueError) as error:
        raise AnalysisError(f"CLI JSON field {field!r} is not an integer: {value!r}") from error


def _require(data: Mapping[str, object], key: str, command: str) -> object:
    if key not in data:
        raise AnalysisError(f"`lawsynth {command}` JSON is missing the {key!r} field")
    return data[key]


def _parse_stability(data: Mapping[str, object]) -> StabilityReport:
    """Parse the ``stability --json`` object (see ``stability.rs::render_json``)."""
    points_raw = _require(data, "fixed_points", "stability")
    if not isinstance(points_raw, list):
        raise AnalysisError("`lawsynth stability` JSON 'fixed_points' must be a list")
    fixed_points = []
    for entry in points_raw:
        eigenvalues = tuple(
            Eigenvalue(re=_as_float(item.get("re"), "eigenvalues[].re"),
                       im=_as_float(item.get("im"), "eigenvalues[].im"))
            for item in entry.get("eigenvalues", [])
        )
        fixed_points.append(
            FixedPoint(
                coordinates=tuple(_as_float(value, "coordinates[]") for value in entry.get("coordinates", [])),
                classification=str(_require(entry, "classification", "stability")),
                inconclusive=bool(entry.get("inconclusive", False)),
                eigenvalues=eigenvalues,
            )
        )
    return StabilityReport(
        world=str(data.get("world", "")),
        states=tuple(str(state) for state in _require(data, "states", "stability")),  # type: ignore[union-attr]
        seeds_total=_as_int(_require(data, "seeds_total", "stability"), "seeds_total"),
        seeds_converged=_as_int(_require(data, "seeds_converged", "stability"), "seeds_converged"),
        fixed_points=tuple(fixed_points),
    )


def _parse_validation(data: Mapping[str, object] | None) -> ControlValidation | None:
    if not data:
        return None
    per_state = tuple(
        StateScore(
            state=str(entry.get("state", "")),
            r_squared=_as_float(entry.get("r_squared"), "validation.per_state[].r_squared"),
            rmse=_as_float(entry.get("rmse"), "validation.per_state[].rmse"),
        )
        for entry in data.get("per_state", [])
    )
    return ControlValidation(
        in_sample=bool(data.get("in_sample", True)),
        per_state=per_state,
        aggregate_r_squared=_as_float(data.get("aggregate_r_squared"), "validation.aggregate_r_squared"),
        aggregate_rmse=_as_float(data.get("aggregate_rmse"), "validation.aggregate_rmse"),
    )


def _parse_controlled(data: Mapping[str, object]) -> ControlledModel:
    """Parse the ``control --json`` object (see ``control.rs::render_json``)."""
    equations_raw = _require(data, "equations", "control")
    if not isinstance(equations_raw, list):
        raise AnalysisError("`lawsynth control` JSON 'equations' must be a list")
    equations = []
    for entry in equations_raw:
        terms = tuple(
            ControlTerm(
                term=str(term.get("term", "")),
                coefficient=_as_float(term.get("coefficient"), "equations[].terms[].coefficient"),
            )
            for term in entry.get("terms", [])
        )
        equations.append(
            ControlEquation(
                state=str(_require(entry, "state", "control")),
                residual_sum_squares=_as_float(
                    entry.get("residual_sum_squares"), "equations[].residual_sum_squares"
                ),
                terms=terms,
            )
        )
    validation = data.get("validation")
    return ControlledModel(
        source=str(data.get("source", "")),
        states=tuple(str(state) for state in _require(data, "states", "control")),  # type: ignore[union-attr]
        controls=tuple(str(control) for control in _require(data, "controls", "control")),  # type: ignore[union-attr]
        equations=tuple(equations),
        validation=_parse_validation(validation if isinstance(validation, Mapping) else None),
    )


# The `domains` (list) and `domains show` subcommands have no `--json` in the
# engine; their text output is stable, so we parse it. `domains run` emits real
# JSON and is decoded with json.loads.
_SHOW_NAME = re.compile(r"^Domain preset:\s*(.+?)\s*$")
_SHOW_LAW = re.compile(r"^\s*d/dt\s+(\S+)\s*=\s*(.+?)\s*$")
_SHOW_STATE_ORDER = re.compile(r"^Reference law \(state order:\s*(.+?)\):\s*$")
_SHOW_DEGREE = re.compile(r"^\s*polynomial degree:\s*(\d+)\s*$")
_SHOW_BOOL = re.compile(r"^\s*(trigonometric|rational):\s*(\S+)\s*$")


def _parse_domain_names(text: str) -> list[str]:
    """Extract preset names from ``lawsynth domains`` (see ``domains.rs::render_list``).

    Names are printed two-space indented; their one-line summaries are four-space
    indented, so a name line starts with exactly two spaces.
    """
    names: list[str] = []
    for line in text.splitlines():
        if line.startswith("  ") and not line.startswith("   ") and line.strip():
            names.append(line.strip())
    return names


def _parse_domain_show(text: str) -> dict[str, object]:
    """Parse ``lawsynth domains show NAME`` text into a structured dict.

    The engine emits no ``--json`` for ``show``; the returned dict carries the
    reliably-parseable fields (name, state order, reference laws, the discovery
    config knobs) plus the verbatim ``text`` so nothing is lost or fabricated.
    """
    result: dict[str, object] = {"text": text}
    reference_laws: dict[str, str] = {}
    discovery: dict[str, object] = {}
    in_config = False
    for line in text.splitlines():
        if (match := _SHOW_NAME.match(line)):
            result["name"] = match.group(1)
        elif (match := _SHOW_STATE_ORDER.match(line)):
            result["state_order"] = [part.strip() for part in match.group(1).split(",")]
        elif line.startswith("Discovery configuration:"):
            in_config = True
        elif (match := _SHOW_LAW.match(line)):
            reference_laws[match.group(1)] = match.group(2)
        elif in_config and (match := _SHOW_DEGREE.match(line)):
            discovery["polynomial_degree"] = int(match.group(1))
        elif in_config and (match := _SHOW_BOOL.match(line)):
            discovery[match.group(1)] = match.group(2).lower() == "true"
    result["reference_laws"] = reference_laws
    if discovery:
        result["discovery"] = discovery
    return result


# --------------------------------------------------------------------------- #
# Binary discovery — mirrors benchmarks/_engine.py exactly                      #
# --------------------------------------------------------------------------- #


def _repository_root() -> Path | None:
    """Walk up from this module and the cwd for the LawSynth repository root."""
    starts = [Path(__file__).resolve(), Path.cwd().resolve()]
    for start in starts:
        for parent in (start, *start.parents):
            if (parent / "Cargo.toml").is_file() and (parent / "crates").is_dir():
                return parent
    return None


def _candidate_binaries() -> list[Path]:
    """The search order for the compiled CLI (matches ``_engine.binary_candidates``)."""
    ordered: list[Path] = []
    override = os.environ.get("LAWSYNTH_BIN")
    if override:
        ordered.append(Path(override))
    root = _repository_root()
    if root is not None:
        ordered.append(root / "target" / "debug" / "lawsynth")
        ordered.append(root / "target" / "release" / "lawsynth")
    return ordered


def _locate_binary() -> Path:
    """Return the first executable CLI binary, or raise :class:`MissingBinaryError`."""
    for candidate in _candidate_binaries():
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return candidate
    raise MissingBinaryError(
        "no compiled lawsynth CLI binary was found. Build it with "
        "`cargo build -p lawsynth-cli` (produces target/debug/lawsynth), or set "
        "LAWSYNTH_BIN to the binary path. The Python analysis client is a "
        "presentation layer over the engine and requires the built CLI."
    )


def _offline_env() -> dict[str, str]:
    """A fixed, locale-pinned environment so the call cannot depend on locale/TZ."""
    env = dict(os.environ)
    env.update({"LC_ALL": "C", "LANG": "C", "LC_NUMERIC": "C", "TZ": "UTC"})
    return env


def _invoke(args: Sequence[str], *, binary: str | PathLike[str] | None = None) -> str:
    """Run the CLI by absolute path and return stdout; raise typed errors on failure.

    Raises :class:`MissingBinaryError` when the binary is absent and
    :class:`CliError` (carrying command + stderr) on a non-zero exit.
    """
    resolved = Path(binary) if binary is not None else _locate_binary()
    command = [str(resolved), *(str(arg) for arg in args)]
    completed = subprocess.run(  # noqa: S603 - fixed argv, no shell, absolute path
        command,
        capture_output=True,
        text=True,
        env=_offline_env(),
        check=False,
    )
    if completed.returncode != 0:
        raise CliError(command=command, returncode=completed.returncode, stderr=completed.stderr)
    return completed.stdout


def _run_cli(args: Sequence[str], *, binary: str | PathLike[str] | None = None) -> dict[str, object]:
    """Locate the binary, run ``args``, and parse the CLI's JSON stdout to a dict."""
    stdout = _invoke(args, binary=binary)
    try:
        parsed = json.loads(stdout)
    except json.JSONDecodeError as error:
        raise AnalysisError(
            f"could not parse `lawsynth {args[0] if args else ''}` JSON output: {error}"
        ) from error
    if not isinstance(parsed, dict):
        raise AnalysisError("expected a JSON object from the lawsynth CLI")
    return parsed


# --------------------------------------------------------------------------- #
# Public API                                                                   #
# --------------------------------------------------------------------------- #


def _format_box(box: str | Sequence[tuple[float, float]]) -> str:
    """Format a search box as ``LOW:HIGH[,LOW:HIGH...]`` (pass-through for strings)."""
    if isinstance(box, str):
        text = box.strip()
        if not text:
            raise ValidationError("box must not be empty")
        return text
    intervals = list(box)
    if not intervals:
        raise ValidationError("box must contain at least one (low, high) interval")
    parts: list[str] = []
    for interval in intervals:
        low, high = interval
        low, high = float(low), float(high)
        if low > high:
            raise ValidationError(f"box interval ({low}, {high}) has lower bound above upper bound")
        parts.append(f"{low}:{high}")
    return ",".join(parts)


def stability(
    world_path: str | PathLike[str],
    *,
    box: str | Sequence[tuple[float, float]],
    grid: int | None = None,
    tolerance: float | None = None,
    dedup: float | None = None,
    marginal_band: float | None = None,
    max_iterations: int | None = None,
    divergence: float | None = None,
) -> StabilityReport:
    """Locate and classify the fixed points of a ``.lsworld`` via the CLI engine.

    Runs ``lawsynth stability WORLD --box ... --json`` and parses the result into
    a :class:`StabilityReport`. ``box`` is the mandatory search box: either the
    raw ``"LOW:HIGH,LOW:HIGH"`` string the CLI expects (one interval per state, in
    state order) or a sequence of ``(low, high)`` pairs. The optional knobs map
    one-to-one onto the CLI flags (``--grid``, ``--tolerance``, ``--dedup``,
    ``--marginal-band``, ``--max-iterations``, ``--divergence``).

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` if the command fails (e.g. the world cannot be read).
    """
    args: list[str] = ["stability", str(world_path), "--box", _format_box(box)]
    if grid is not None:
        args += ["--grid", str(int(grid))]
    if tolerance is not None:
        args += ["--tolerance", repr(float(tolerance))]
    if dedup is not None:
        args += ["--dedup", repr(float(dedup))]
    if marginal_band is not None:
        args += ["--marginal-band", repr(float(marginal_band))]
    if max_iterations is not None:
        args += ["--max-iterations", str(int(max_iterations))]
    if divergence is not None:
        args += ["--divergence", repr(float(divergence))]
    args.append("--json")
    return _parse_stability(_run_cli(args))


def discover_controlled(
    obs_csv: str | PathLike[str],
    *,
    states: Sequence[str],
    controls: Sequence[str],
    time: str = "time",
    degree: int | None = None,
    threshold: float | None = None,
    validate: bool = False,
) -> ControlledModel:
    """Discover a forced model ``dx/dt = f(x, u)`` (SINDYc) via the CLI engine.

    Runs ``lawsynth control OBSERVATIONS --time ... --state ... --control ...
    --json`` and parses the fitted per-state equations into a
    :class:`ControlledModel`. ``states``/``controls`` name the observation columns
    (state columns are differentiated and predicted; control columns enter the
    library but are never predicted). ``time`` is the time column (default
    ``"time"``). With ``validate=True`` the model is rolled forward under the
    dataset's own controls and the (in-sample) per-state and aggregate R²/RMSE are
    attached as ``model.validation``.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit.
    """
    state_list = [str(name) for name in states]
    control_list = [str(name) for name in controls]
    if not state_list:
        raise ValidationError("at least one state column is required")
    if not control_list:
        raise ValidationError("at least one control column is required")
    args: list[str] = [
        "control", str(obs_csv),
        "--time", str(time),
        "--state", ",".join(state_list),
        "--control", ",".join(control_list),
    ]
    if degree is not None:
        args += ["--degree", str(int(degree))]
    if threshold is not None:
        args += ["--threshold", repr(float(threshold))]
    if validate:
        args.append("--validate")
    args.append("--json")
    return _parse_controlled(_run_cli(args))


def domains() -> list[str]:
    """List the curated domain preset names via ``lawsynth domains``.

    The engine's list output has no ``--json``; its text is stable and parsed
    here. Raises :class:`MissingBinaryError` if the CLI is not built.
    """
    return _parse_domain_names(_invoke(["domains"]))


def domain_show(name: str) -> dict[str, object]:
    """Show a preset's reference law and discovery config via ``lawsynth domains show``.

    Returns a structured dict (name, state order, reference laws, discovery
    config) plus the verbatim ``text``. Raises :class:`CliError` for an unknown
    preset and :class:`MissingBinaryError` if the CLI is not built.
    """
    return _parse_domain_show(_invoke(["domains", "show", str(name)]))


def domain_run(name: str) -> dict[str, object]:
    """Run a preset's round-trip recovery via ``lawsynth domains run NAME --json``.

    Returns the CLI's JSON object (``preset``, ``recovered``, ``tolerance``,
    ``laws``, and per-state ``recovery`` with ``rhs_rmse`` /
    ``discovered_terms`` / ``reference_terms``). The round-trip runs on clean
    synthetic data — a high score validates the preset's search space, not
    robustness to real noise. Raises :class:`CliError` for an unknown preset.
    """
    return _run_cli(["domains", "run", str(name), "--json"])


# --------------------------------------------------------------------------- #
# Attach convenience methods to DiscoveryResult / Study (best-effort, lazy)     #
# --------------------------------------------------------------------------- #


def _install() -> None:
    """Attach ``.stability(...)`` to Study / DiscoveryResult (mirrors export/tracking).

    The stability CLI reads a ``.lsworld`` bundle, so the convenience method saves
    the in-memory discovered world to a temporary bundle and analyses that.
    """
    try:
        from .study import DiscoveryResult, Study
    except Exception:  # pragma: no cover - defensive at import time
        return

    def _stability_method(self: object, **kwargs: object) -> StabilityReport:
        import tempfile

        with tempfile.TemporaryDirectory(prefix="lawsynth-stability-") as tmp:
            target = Path(tmp) / "world.lsworld"
            self.save(target)  # type: ignore[attr-defined]
            return stability(target, **kwargs)  # type: ignore[arg-type]

    for cls in (Study, DiscoveryResult):
        if not hasattr(cls, "stability"):
            cls.stability = _stability_method  # type: ignore[attr-defined]


_install()
