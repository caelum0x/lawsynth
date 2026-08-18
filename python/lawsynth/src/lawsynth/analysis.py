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
    "Bifurcation",
    "BifurcationReport",
    "Sensitivity",
    "SensitivityReport",
    "EstimateReport",
    "ReducedSystem",
    "ReductionReport",
    "LyapunovReport",
    "Attractor",
    "BasinReport",
    "NetworkEdge",
    "NetworkModel",
    "MpcResult",
    "DiscreteEigenvalue",
    "KoopmanReport",
    "SdeTerm",
    "SdeLaw",
    "SdeBin",
    "SdeState",
    "SdeReport",
    "PdeTerm",
    "PdeReport",
    "Invariant",
    "InvariantReport",
    "Skipped",
    "AnalyzeReport",
    "stability",
    "discover_controlled",
    "domains",
    "domain_show",
    "domain_run",
    "bifurcation",
    "sensitivity",
    "estimate",
    "reduce",
    "lyapunov",
    "basins",
    "network",
    "mpc",
    "koopman",
    "sde",
    "pde",
    "invariants",
    "analyze",
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


@dataclass(frozen=True, slots=True)
class Bifurcation:
    """One detected bifurcation on a continuation branch.

    ``parameter_value`` is the swept parameter's value ``μ*`` at the crossing;
    ``kind`` is the engine's stable token (``"fold"`` for a real eigenvalue
    through zero — saddle-node / transcritical / pitchfork — or ``"hopf"`` for a
    complex pair crossing). ``fixed_point`` is ordered like the report's
    ``states``; ``eigenvalue`` is the Jacobian eigenvalue on the imaginary axis.
    """

    parameter_value: float
    kind: str
    branch_id: int
    fixed_point: tuple[float, ...]
    eigenvalue: Eigenvalue

    def at(self, states: Sequence[str]) -> dict[str, float]:
        """Map the bifurcation's fixed point onto ``states`` (``{state: value}``)."""
        return {str(name): value for name, value in zip(states, self.fixed_point)}


@dataclass(frozen=True, slots=True)
class BifurcationReport:
    """The parsed result of ``lawsynth bifurcation WORLD --parameter ... --json``.

    Continuation sweeps a free parameter across ``[range_min, range_max]`` in
    ``steps`` grid points, re-locating the fixed points at each value and
    stitching them into ``branch_count`` branches. An empty ``bifurcations``
    means the fixed points kept their stability across the whole range, not that
    the system cannot bifurcate — widen the range or refine the steps. The named
    parameter must actually appear in a law (a discovered world that inlines its
    coefficients as constants has no free parameter, and the CLI says so via a
    :class:`CliError`).
    """

    world: str
    states: tuple[str, ...]
    parameter: str
    range_min: float
    range_max: float
    steps: int
    branch_count: int
    bifurcations: tuple[Bifurcation, ...]


@dataclass(frozen=True, slots=True)
class Sensitivity:
    """One trajectory sensitivity ``∂state/∂parameter`` at the final time."""

    state: str
    parameter: str
    value: float


@dataclass(frozen=True, slots=True)
class SensitivityReport:
    """The parsed result of ``lawsynth sensitivity WORLD --parameters ... --json``.

    Forward (variational) sensitivities ``∂x_i/∂θ_j`` at ``final_time`` for each
    ``state`` × ``parameter`` pair. Each requested parameter must be a declared
    world parameter (its value is read from the world); a parameter that never
    appears in the laws differentiates to exactly zero (the correct, non-fabricated
    answer). ``sensitivities`` is flattened in state-major, parameter-minor order.
    """

    world: str
    states: tuple[str, ...]
    parameters: tuple[str, ...]
    final_time: float
    sensitivities: tuple[Sensitivity, ...]

    def value(self, state: str, parameter: str) -> float:
        """Return ``∂state/∂parameter`` at the final time, or raise :class:`KeyError`."""
        for entry in self.sensitivities:
            if entry.state == state and entry.parameter == parameter:
                return entry.value
        raise KeyError(f"no sensitivity for state {state!r} / parameter {parameter!r}")

    def matrix(self) -> tuple[tuple[float, ...], ...]:
        """The sensitivity matrix as rows over ``states`` and columns over ``parameters``."""
        return tuple(
            tuple(self.value(state, parameter) for parameter in self.parameters)
            for state in self.states
        )


@dataclass(frozen=True, slots=True)
class EstimateReport:
    """The parsed result of ``lawsynth estimate WORLD --box ... --measure ... --json``.

    The world's field is linearized at the **first** fixed point inside ``--box``
    (``A = ∂f/∂x`` there; ``fixed_points_found`` reports how many were located so
    the choice is auditable), ``measured`` states form the output map ``C``, and a
    state estimator is designed. ``method`` is ``"pole_placement"`` (Ackermann,
    single measured state) or ``"kalman"`` (steady-state, several measured states).
    ``gain`` is the observer gain ``L`` (rows over states, columns over outputs);
    ``error_poles`` are the eigenvalues of ``A − L C`` and ``convergent`` is true
    when all have negative real part (the estimate ``x̂ → x``). ``covariance`` is the
    steady-state error covariance ``P`` for the Kalman design, else ``None``.
    """

    world: str
    states: tuple[str, ...]
    fixed_point: tuple[float, ...]
    fixed_points_found: int
    measured: tuple[str, ...]
    method: str
    gain: tuple[tuple[float, ...], ...]
    error_poles: tuple[Eigenvalue, ...]
    convergent: bool
    covariance: tuple[tuple[float, ...], ...] | None


@dataclass(frozen=True, slots=True)
class ReducedSystem:
    """The reduced linear system ``(A, B, C)`` from balanced truncation.

    Each matrix is stored row-major as a tuple of row tuples.
    """

    a: tuple[tuple[float, ...], ...]
    b: tuple[tuple[float, ...], ...]
    c: tuple[tuple[float, ...], ...]


@dataclass(frozen=True, slots=True)
class ReductionReport:
    """The parsed result of ``lawsynth reduce WORLD --box ... --json``.

    The world's field is linearized at the first fixed point inside ``--box``
    (``A = ∂f/∂x``) and reduced by balanced truncation. Balanced truncation
    requires a *stable* (Hurwitz) fixed point; an unstable equilibrium is rejected
    by the engine with a clear :class:`CliError`. ``hankel_singular_values`` are
    non-increasing; ``order`` is the retained order; ``error_bound`` is the
    ``H∞`` bound ``‖G − Gr‖∞``; ``reduced`` carries the reduced ``A/B/C``.
    ``measured`` is the selected output states, or ``None`` when ``C = I``.
    """

    world: str
    states: tuple[str, ...]
    fixed_point: tuple[float, ...]
    measured: tuple[str, ...] | None
    hankel_singular_values: tuple[float, ...]
    order: int
    error_bound: float
    reduced: ReducedSystem


@dataclass(frozen=True, slots=True)
class LyapunovReport:
    """The parsed result of ``lawsynth lyapunov WORLD --initial ... --json``.

    The world's laws are read as an autonomous field ``ẋ = f(x)`` and the
    deterministic Benettin/QR estimator is run from ``--initial``. This is a
    **time-averaged estimate**, not an exact spectrum: accuracy grows with the run
    length (``steps``) and shrinks with the step (``dt``), and the initial
    condition should lie in the basin of the target attractor. The ``sum`` (the
    time-averaged divergence) is the tightest quantity.

    ``exponents`` is the spectrum sorted descending; ``largest`` its first entry;
    ``sum`` the exponent sum; ``kaplan_yorke_dimension`` the Kaplan–Yorke (Lyapunov)
    dimension; ``integration_time`` the post-transient averaging window; and
    ``chaotic`` is exactly ``largest > 0`` (the signature of chaos).
    """

    world: str
    states: tuple[str, ...]
    exponents: tuple[float, ...]
    largest: float
    sum: float
    kaplan_yorke_dimension: float
    integration_time: float
    chaotic: bool


@dataclass(frozen=True, slots=True)
class Attractor:
    """One stable fixed-point attractor located by a basin mapping.

    ``coordinates`` are ordered like the report's ``states``; ``classification`` is
    the engine's human label (``"stable node"`` / ``"stable spiral"``); and
    ``basin_fraction`` is this attractor's share of the *settled* initial
    conditions (the fractions over all attractors sum to ``1`` up to rounding).
    """

    coordinates: tuple[float, ...]
    classification: str
    basin_fraction: float

    def at(self, states: Sequence[str]) -> dict[str, float]:
        """Map coordinates onto ``states`` (``{state: value}``)."""
        return {str(name): value for name, value in zip(states, self.coordinates)}


@dataclass(frozen=True, slots=True)
class BasinReport:
    """The parsed result of ``lawsynth basins WORLD --box ... --json``.

    A deterministic grid of initial conditions (``resolution`` samples per axis,
    ``total`` in all) is integrated forward with fixed-step RK4 and each is
    classified. **Only fixed-point attractors are recognized**: a limit cycle or
    strange attractor reads as ``undetermined`` rather than being forced into a
    basin, and a trajectory leaving the box is ``escaped``. ``settled`` is the
    number that reached some attractor. ``grid_labels`` is the flattened per-cell
    fate — ``"a{index}"`` for attractor ``index``, ``"escaped"``, or
    ``"undetermined"``. (The engine's ``--json`` reports the grid ``resolution``
    but not the ``--box`` echo.)
    """

    world: str
    states: tuple[str, ...]
    resolution: int
    total: int
    settled: int
    escaped: int
    undetermined: int
    attractors: tuple[Attractor, ...]
    grid_labels: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class NetworkEdge:
    """One directed coupling edge ``driver -> target`` with its aggregate strength.

    ``driver`` and ``target`` are node *names*; the edge means node ``driver``
    appears in node ``target``'s discovered equation (``driver`` drives
    ``target``). Self edges (``driver == target``) are included.
    """

    driver: str
    target: str
    strength: float


@dataclass(frozen=True, slots=True)
class NetworkModel:
    """The parsed result of ``lawsynth network OBS --state ... --json``.

    Each named column is a node; its derivative is sparsely regressed onto a shared
    polynomial library over all nodes, and a surviving cross term ``x_driver`` in
    node ``target``'s equation is a directed edge ``driver -> target``. The graph
    is **correlational, not causal**: a confounder or common drive can induce a
    spurious edge, and only couplings the library can represent and that clear the
    edge threshold are recovered.

    ``adjacency`` is the boolean matrix with ``adjacency[target][driver]`` true
    when ``driver -> target``; ``strength`` is the matching aggregated-strength
    matrix (same ``[target][driver]`` indexing); ``edges`` is the flattened edge
    list in ascending ``(target, driver)`` order.
    """

    source: str
    nodes: tuple[str, ...]
    adjacency: tuple[tuple[bool, ...], ...]
    strength: tuple[tuple[float, ...], ...]
    edges: tuple[NetworkEdge, ...]

    def edge_strength(self, driver: str, target: str) -> float:
        """Return the aggregated strength of ``driver -> target`` via node names.

        Uses the ``strength`` matrix indexed as ``strength[target][driver]``.
        Raises :class:`KeyError` if either name is not a node.
        """
        index = {name: position for position, name in enumerate(self.nodes)}
        try:
            return self.strength[index[target]][index[driver]]
        except KeyError as error:
            raise KeyError(f"unknown node {error.args[0]!r}") from error


@dataclass(frozen=True, slots=True)
class MpcResult:
    """The parsed result of ``lawsynth mpc WORLD --control ... --setpoint ... --json``.

    The world's forced field ``ẋ = f(x, u)`` is regulated to ``setpoint`` by
    **successive-linearization LQR-MPC**: at each step the engine relinearizes,
    designs a local LQR gain, applies the first (optionally saturated) move, and
    RK4-advances the true nonlinear plant. Honest limits: the local LQR needs a
    **stabilizable** linearization, saturation is a clamp (not a constraint-optimal
    projection), optimality is only local to each linearization, and there is no
    horizon/feasibility guarantee — a failed LQR design surfaces as a
    :class:`CliError`.

    ``setpoint`` / ``final_state`` are ordered like ``states``; ``final_error_norm``
    is the Euclidean state error at the final step (``None`` if unavailable);
    ``state_trajectory`` / ``control_trajectory`` are the per-step rows over
    ``states`` / ``controls``.
    """

    world: str
    states: tuple[str, ...]
    controls: tuple[str, ...]
    setpoint: tuple[float, ...]
    final_state: tuple[float, ...]
    final_error_norm: float | None
    state_trajectory: tuple[tuple[float, ...], ...]
    control_trajectory: tuple[tuple[float, ...], ...]


# --------------------------------------------------------------------------- #
# Discovery-engine results — koopman / sde / pde                                #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class DiscreteEigenvalue:
    """A discrete-time DMD eigenvalue ``λ = re + im i`` with its modulus ``|λ|``.

    ``modulus`` is the per-step growth factor: ``|λ| < 1`` decays, ``> 1`` grows,
    ``≈ 1`` is neutral. It is a superset of :class:`Eigenvalue` (the ``koopman``
    JSON emits ``modulus`` alongside ``re``/``im`` for the discrete spectrum only;
    the continuous spectrum is a plain ``Eigenvalue``).
    """

    re: float
    im: float
    modulus: float


@dataclass(frozen=True, slots=True)
class KoopmanReport:
    """The parsed result of ``lawsynth koopman DATASET --state ... --json`` (DMD).

    Dynamic Mode Decomposition fits the best-fit **linear operator** ``A`` that
    advances the state one step (``x' ≈ A x``) over the dataset's snapshot pairs,
    and reports its spectrum. This is honestly a **linear (or lifted-linear)
    approximation** of the dynamics, not a symbolic nonlinear law, and no
    ``.lsworld`` bundle is written.

    ``states`` is the operator's row/column order (the dataset's schema order).
    ``rank`` is the (optionally truncated) SVD rank actually used and
    ``singular_values`` the retained singular values (non-increasing). The
    ``discrete_eigenvalues`` are ``λ`` (per step, each carrying its ``modulus``);
    the ``continuous_eigenvalues`` are ``μ = ln(λ)/Δt`` (``re`` = growth rate,
    ``im`` = angular frequency). ``spectral_radius`` is ``ρ = max|λ|`` and
    ``stable`` is exactly ``ρ < 1`` (asymptotic stability of the linear operator).
    """

    source: str
    states: tuple[str, ...]
    rank: int
    dt: float
    singular_values: tuple[float, ...]
    discrete_eigenvalues: tuple[DiscreteEigenvalue, ...]
    continuous_eigenvalues: tuple[Eigenvalue, ...]
    spectral_radius: float
    stable: bool


@dataclass(frozen=True, slots=True)
class SdeTerm:
    """One active polynomial term ``coefficient * label`` (``label`` is ``x^power``)."""

    label: str
    power: int
    coefficient: float


@dataclass(frozen=True, slots=True)
class SdeLaw:
    """A discovered drift or diffusion law ``Σ coefficient·label`` with its residual."""

    terms: tuple[SdeTerm, ...]
    residual_sum_squares: float

    def expression(self) -> str:
        """Render the law as ``c0*l0 + c1*l1 + ...`` (``"0"`` if every term dropped)."""
        if not self.terms:
            return "0"
        return " + ".join(f"{term.coefficient}*{term.label}" for term in self.terms)


@dataclass(frozen=True, slots=True)
class SdeBin:
    """One binned Kramers–Moyal conditional-moment row for a state.

    ``x_center`` is the bin's state-space center; ``drift`` / ``diffusion`` are the
    estimated first / second conditional moments there; ``count`` is the bin
    occupancy (low-occupancy bins are statistically unreliable).
    """

    x_center: float
    drift: float
    diffusion: float
    count: int


@dataclass(frozen=True, slots=True)
class SdeState:
    """The recovered drift/diffusion laws and binned table for one state.

    ``drift`` is ``a(x)`` and ``diffusion`` is ``b²(x)`` in ``dX = a(X) dt +
    b(X) dW``. ``trusted_bins`` is how many of ``bins`` cleared the minimum
    occupancy and actually fed the sparse regression.
    """

    state: str
    trusted_bins: int
    drift: SdeLaw
    diffusion: SdeLaw
    bins: tuple[SdeBin, ...]


@dataclass(frozen=True, slots=True)
class SdeReport:
    """The parsed result of ``lawsynth sde OBS --state ... --json``.

    A **statistical** (Kramers–Moyal) estimator: from a single noisy sample path
    of a diagonal-noise Itô SDE it estimates the drift ``a(x)`` and diffusion
    ``b²(x)`` per state by binning conditional moments and sparse-regressing each
    onto a polynomial library. Accuracy grows with path length and rarely-visited
    (low-occupancy) bins are unreliable; no ``.lsworld`` bundle is written.

    ``dt`` is the mean increment and ``increments`` the number of increments used.
    """

    source: str
    dt: float
    increments: int
    states: tuple[SdeState, ...]


@dataclass(frozen=True, slots=True)
class PdeTerm:
    """One PDE-FIND library term ``coefficient · u^u_power · D_derivative_order``.

    ``label`` is the engine's stable rendering (e.g. ``"u_xx"``, ``"u^2 u_x"``);
    ``u_power`` is the field power and ``derivative_order`` the spatial-derivative
    order of the term.
    """

    label: str
    u_power: int
    derivative_order: int
    coefficient: float


@dataclass(frozen=True, slots=True)
class PdeReport:
    """The parsed result of ``lawsynth pde FIELD --dx DX --dt DT --json`` (PDE-FIND).

    From snapshots of a 1-D field ``u(x, t)`` on a regular space–time grid, the
    evolution law ``u_t = F(u, u_x, u_xx, …)`` is recovered by central **finite
    differences** plus sparse regression. Finite differencing amplifies noise, so
    recovery is noise-sensitive and needs a resolved grid (finer grids tighten the
    coefficients); no ``.lsworld`` bundle is written.

    ``variable`` is the field name (``"u"``); ``time_snapshots`` × ``spatial_points``
    is the grid shape and ``interior_points`` the count that fed the regression
    (the boundary is dropped by central differencing). ``dx`` / ``dt`` echo the
    grid steps, ``law`` is the rendered ``u_t = ...`` string, and ``terms`` are the
    surviving library terms with their coefficients.
    """

    source: str
    variable: str
    time_snapshots: int
    spatial_points: int
    dx: float
    dt: float
    interior_points: int
    residual_sum_squares: float
    law: str
    terms: tuple[PdeTerm, ...]


@dataclass(frozen=True, slots=True)
class Invariant:
    """One detected conserved quantity ``H(x)`` over the candidate basis.

    ``combination`` is the engine's readable rendering (coefficients rescaled so the
    largest-magnitude term reads ``1.00``, e.g. ``"1.00·v^2 + 1.00·x^2"``);
    ``coefficients`` is the raw unit-norm weight vector over the report's
    ``basis_labels`` (same order); ``residual`` is ``‖L_f H‖`` — how close the Lie
    derivative ``∇H·f`` is to zero on the sample grid — and ``singular_value`` the
    matching near-null singular value. Both are near zero for a genuine invariant.
    """

    combination: str
    coefficients: tuple[float, ...]
    residual: float
    singular_value: float


@dataclass(frozen=True, slots=True)
class InvariantReport:
    """The parsed result of ``lawsynth invariants WORLD --json``.

    The world's laws are read as an autonomous field ``ẋ = f(x)`` (every declared
    parameter pinned at its stored value) and conserved quantities are sought as
    combinations of a candidate library: monomials up to total ``degree``
    (``trigonometric`` adds sin/cos terms), whose readable names are
    ``basis_labels``. The Lie-derivative matrix is sampled on a deterministic grid
    of ``resolution`` points per axis inside ``sample_box`` (the shared per-axis
    interval ``(lo, hi)``) and each near-null direction within ``tolerance`` is
    reported as an :class:`Invariant`.

    The search is **library-bounded**: an empty ``invariants`` means no conserved
    quantity expressible in the degree-``D`` library was found within tolerance —
    not that the system conserves nothing. Raise ``degree``, add ``trig``, or widen
    the box to search a richer library.
    """

    world: str
    degree: int
    trigonometric: bool
    sample_box: tuple[float, float]
    resolution: int
    tolerance: float
    basis_labels: tuple[str, ...]
    invariants: tuple[Invariant, ...]


@dataclass(frozen=True, slots=True)
class Skipped:
    """A sub-analysis of :func:`analyze` that could not run, with an honest note.

    ``lawsynth analyze`` runs stability, Lyapunov, and invariants in one pass; a
    part that cannot run (a non-autonomous world, a Lyapunov run that diverges, ...)
    is reported as this small skip marker rather than failing the whole command.
    ``note`` is the engine's human-readable reason.
    """

    note: str


@dataclass(frozen=True, slots=True)
class AnalyzeReport:
    """The parsed result of ``lawsynth analyze WORLD --box ... --json``.

    A one-shot dynamics summary combining three engines on one world. Each
    sub-object is byte-for-byte the shape the matching standalone command emits, so
    the fields are the very same dataclasses those commands' parsers return:
    ``stability`` a :class:`StabilityReport`, ``lyapunov`` a :class:`LyapunovReport`,
    ``invariants`` an :class:`InvariantReport` — or a :class:`Skipped` marker when
    that part could not run for this world. ``world`` and ``states`` echo the
    analyzed world (states in the engine's schema order).
    """

    world: str
    states: tuple[str, ...]
    stability: "StabilityReport | Skipped"
    lyapunov: "LyapunovReport | Skipped"
    invariants: "InvariantReport | Skipped"


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


def _parse_eigenvalue(item: object, field: str) -> Eigenvalue:
    """Parse a ``{"re": .., "im": ..}`` object into an :class:`Eigenvalue`."""
    if not isinstance(item, Mapping):
        raise AnalysisError(f"CLI JSON field {field!r} must be an object with 're'/'im'")
    return Eigenvalue(
        re=_as_float(item.get("re"), f"{field}.re"),
        im=_as_float(item.get("im"), f"{field}.im"),
    )


def _as_matrix(value: object, field: str) -> tuple[tuple[float, ...], ...]:
    """Parse a row-major list-of-lists (``matrix_json``) into nested float tuples."""
    if not isinstance(value, list):
        raise AnalysisError(f"CLI JSON field {field!r} must be a list of rows")
    rows: list[tuple[float, ...]] = []
    for row_index, row in enumerate(value):
        if not isinstance(row, list):
            raise AnalysisError(f"CLI JSON field {field!r}[{row_index}] must be a list")
        rows.append(tuple(_as_float(cell, f"{field}[{row_index}][]") for cell in row))
    return tuple(rows)


def _as_bool_matrix(value: object, field: str) -> tuple[tuple[bool, ...], ...]:
    """Parse a row-major list-of-lists of booleans into nested bool tuples."""
    if not isinstance(value, list):
        raise AnalysisError(f"CLI JSON field {field!r} must be a list of rows")
    rows: list[tuple[bool, ...]] = []
    for row_index, row in enumerate(value):
        if not isinstance(row, list):
            raise AnalysisError(f"CLI JSON field {field!r}[{row_index}] must be a list")
        rows.append(tuple(bool(cell) for cell in row))
    return tuple(rows)


def _parse_bifurcation(data: Mapping[str, object]) -> BifurcationReport:
    """Parse the ``bifurcation --json`` object (see ``bifurcation.rs::render_json``)."""
    range_raw = _require(data, "range", "bifurcation")
    if not isinstance(range_raw, Mapping):
        raise AnalysisError("`lawsynth bifurcation` JSON 'range' must be an object")
    bifurcations_raw = _require(data, "bifurcations", "bifurcation")
    if not isinstance(bifurcations_raw, list):
        raise AnalysisError("`lawsynth bifurcation` JSON 'bifurcations' must be a list")
    bifurcations = tuple(
        Bifurcation(
            parameter_value=_as_float(
                _require(entry, "parameter_value", "bifurcation"), "bifurcations[].parameter_value"
            ),
            kind=str(_require(entry, "kind", "bifurcation")),
            branch_id=_as_int(_require(entry, "branch_id", "bifurcation"), "bifurcations[].branch_id"),
            fixed_point=tuple(
                _as_float(value, "bifurcations[].fixed_point[]")
                for value in entry.get("fixed_point", [])
            ),
            eigenvalue=_parse_eigenvalue(
                _require(entry, "eigenvalue", "bifurcation"), "bifurcations[].eigenvalue"
            ),
        )
        for entry in bifurcations_raw
    )
    return BifurcationReport(
        world=str(data.get("world", "")),
        states=tuple(str(state) for state in _require(data, "states", "bifurcation")),  # type: ignore[union-attr]
        parameter=str(_require(data, "parameter", "bifurcation")),
        range_min=_as_float(_require(range_raw, "min", "bifurcation"), "range.min"),
        range_max=_as_float(_require(range_raw, "max", "bifurcation"), "range.max"),
        steps=_as_int(_require(data, "steps", "bifurcation"), "steps"),
        branch_count=_as_int(_require(data, "branch_count", "bifurcation"), "branch_count"),
        bifurcations=bifurcations,
    )


def _parse_sensitivity(data: Mapping[str, object]) -> SensitivityReport:
    """Parse the ``sensitivity --json`` object (see ``sensitivity.rs::render_json``)."""
    entries_raw = _require(data, "sensitivities", "sensitivity")
    if not isinstance(entries_raw, list):
        raise AnalysisError("`lawsynth sensitivity` JSON 'sensitivities' must be a list")
    sensitivities = tuple(
        Sensitivity(
            state=str(_require(entry, "state", "sensitivity")),
            parameter=str(_require(entry, "parameter", "sensitivity")),
            value=_as_float(_require(entry, "value", "sensitivity"), "sensitivities[].value"),
        )
        for entry in entries_raw
    )
    return SensitivityReport(
        world=str(data.get("world", "")),
        states=tuple(str(state) for state in _require(data, "states", "sensitivity")),  # type: ignore[union-attr]
        parameters=tuple(str(name) for name in _require(data, "parameters", "sensitivity")),  # type: ignore[union-attr]
        final_time=_as_float(_require(data, "final_time", "sensitivity"), "final_time"),
        sensitivities=sensitivities,
    )


def _parse_estimate(data: Mapping[str, object]) -> EstimateReport:
    """Parse the ``estimate --json`` object (see ``estimate.rs::render_json``)."""
    poles_raw = _require(data, "error_poles", "estimate")
    if not isinstance(poles_raw, list):
        raise AnalysisError("`lawsynth estimate` JSON 'error_poles' must be a list")
    covariance = data.get("covariance")
    return EstimateReport(
        world=str(data.get("world", "")),
        states=tuple(str(state) for state in _require(data, "states", "estimate")),  # type: ignore[union-attr]
        fixed_point=tuple(
            _as_float(value, "fixed_point[]") for value in _require(data, "fixed_point", "estimate")  # type: ignore[union-attr]
        ),
        fixed_points_found=_as_int(
            _require(data, "fixed_points_found", "estimate"), "fixed_points_found"
        ),
        measured=tuple(str(name) for name in _require(data, "measured", "estimate")),  # type: ignore[union-attr]
        method=str(_require(data, "method", "estimate")),
        gain=_as_matrix(_require(data, "gain", "estimate"), "gain"),
        error_poles=tuple(
            _parse_eigenvalue(item, "error_poles[]") for item in poles_raw
        ),
        convergent=bool(_require(data, "convergent", "estimate")),
        covariance=_as_matrix(covariance, "covariance") if covariance is not None else None,
    )


def _parse_reduce(data: Mapping[str, object]) -> ReductionReport:
    """Parse the ``reduce --json`` object (see ``reduce.rs::render_json``)."""
    sigma_raw = _require(data, "hankel_singular_values", "reduce")
    if not isinstance(sigma_raw, list):
        raise AnalysisError("`lawsynth reduce` JSON 'hankel_singular_values' must be a list")
    reduced_raw = _require(data, "reduced", "reduce")
    if not isinstance(reduced_raw, Mapping):
        raise AnalysisError("`lawsynth reduce` JSON 'reduced' must be an object")
    measured = data.get("measured")
    return ReductionReport(
        world=str(data.get("world", "")),
        states=tuple(str(state) for state in _require(data, "states", "reduce")),  # type: ignore[union-attr]
        fixed_point=tuple(
            _as_float(value, "fixed_point[]") for value in _require(data, "fixed_point", "reduce")  # type: ignore[union-attr]
        ),
        measured=tuple(str(name) for name in measured) if isinstance(measured, list) else None,
        hankel_singular_values=tuple(
            _as_float(value, "hankel_singular_values[]") for value in sigma_raw
        ),
        order=_as_int(_require(data, "order", "reduce"), "order"),
        error_bound=_as_float(_require(data, "error_bound", "reduce"), "error_bound"),
        reduced=ReducedSystem(
            a=_as_matrix(_require(reduced_raw, "a", "reduce"), "reduced.a"),
            b=_as_matrix(_require(reduced_raw, "b", "reduce"), "reduced.b"),
            c=_as_matrix(_require(reduced_raw, "c", "reduce"), "reduced.c"),
        ),
    )


def _parse_lyapunov(data: Mapping[str, object]) -> LyapunovReport:
    """Parse the ``lyapunov --json`` object (see ``lyapunov.rs::render_json``)."""
    exponents_raw = _require(data, "exponents", "lyapunov")
    if not isinstance(exponents_raw, list):
        raise AnalysisError("`lawsynth lyapunov` JSON 'exponents' must be a list")
    return LyapunovReport(
        world=str(data.get("world", "")),
        states=tuple(str(state) for state in _require(data, "states", "lyapunov")),  # type: ignore[union-attr]
        exponents=tuple(_as_float(value, "exponents[]") for value in exponents_raw),
        largest=_as_float(_require(data, "largest", "lyapunov"), "largest"),
        sum=_as_float(_require(data, "sum", "lyapunov"), "sum"),
        kaplan_yorke_dimension=_as_float(
            _require(data, "kaplan_yorke_dimension", "lyapunov"), "kaplan_yorke_dimension"
        ),
        integration_time=_as_float(
            _require(data, "integration_time", "lyapunov"), "integration_time"
        ),
        chaotic=bool(_require(data, "chaotic", "lyapunov")),
    )


def _parse_basins(data: Mapping[str, object]) -> BasinReport:
    """Parse the ``basins --json`` object (see ``basins.rs::render_json``)."""
    attractors_raw = _require(data, "attractors", "basins")
    if not isinstance(attractors_raw, list):
        raise AnalysisError("`lawsynth basins` JSON 'attractors' must be a list")
    labels_raw = _require(data, "grid_labels", "basins")
    if not isinstance(labels_raw, list):
        raise AnalysisError("`lawsynth basins` JSON 'grid_labels' must be a list")
    attractors = tuple(
        Attractor(
            coordinates=tuple(
                _as_float(value, "attractors[].coordinates[]")
                for value in entry.get("coordinates", [])
            ),
            classification=str(_require(entry, "classification", "basins")),
            basin_fraction=_as_float(
                _require(entry, "basin_fraction", "basins"), "attractors[].basin_fraction"
            ),
        )
        for entry in attractors_raw
    )
    return BasinReport(
        world=str(data.get("world", "")),
        states=tuple(str(state) for state in _require(data, "states", "basins")),  # type: ignore[union-attr]
        resolution=_as_int(_require(data, "resolution", "basins"), "resolution"),
        total=_as_int(_require(data, "total", "basins"), "total"),
        settled=_as_int(_require(data, "settled", "basins"), "settled"),
        escaped=_as_int(_require(data, "escaped", "basins"), "escaped"),
        undetermined=_as_int(_require(data, "undetermined", "basins"), "undetermined"),
        attractors=attractors,
        grid_labels=tuple(str(label) for label in labels_raw),
    )


def _parse_network(data: Mapping[str, object]) -> NetworkModel:
    """Parse the ``network --json`` object (see ``network.rs::render_json``)."""
    edges_raw = _require(data, "edges", "network")
    if not isinstance(edges_raw, list):
        raise AnalysisError("`lawsynth network` JSON 'edges' must be a list")
    edges = tuple(
        NetworkEdge(
            driver=str(_require(entry, "driver", "network")),
            target=str(_require(entry, "target", "network")),
            strength=_as_float(_require(entry, "strength", "network"), "edges[].strength"),
        )
        for entry in edges_raw
    )
    return NetworkModel(
        source=str(data.get("source", "")),
        nodes=tuple(str(node) for node in _require(data, "nodes", "network")),  # type: ignore[union-attr]
        adjacency=_as_bool_matrix(_require(data, "adjacency", "network"), "adjacency"),
        strength=_as_matrix(_require(data, "strength", "network"), "strength"),
        edges=edges,
    )


def _parse_mpc(data: Mapping[str, object]) -> MpcResult:
    """Parse the ``mpc --json`` object (see ``mpc.rs::render_json``)."""
    error_norm = data.get("final_error_norm")
    return MpcResult(
        world=str(data.get("world", "")),
        states=tuple(str(state) for state in _require(data, "states", "mpc")),  # type: ignore[union-attr]
        controls=tuple(str(control) for control in _require(data, "controls", "mpc")),  # type: ignore[union-attr]
        setpoint=tuple(
            _as_float(value, "setpoint[]") for value in _require(data, "setpoint", "mpc")  # type: ignore[union-attr]
        ),
        final_state=tuple(
            _as_float(value, "final_state[]") for value in _require(data, "final_state", "mpc")  # type: ignore[union-attr]
        ),
        final_error_norm=_as_float(error_norm, "final_error_norm") if error_norm is not None else None,
        state_trajectory=_as_matrix(
            _require(data, "state_trajectory", "mpc"), "state_trajectory"
        ),
        control_trajectory=_as_matrix(
            _require(data, "control_trajectory", "mpc"), "control_trajectory"
        ),
    )


def _parse_discrete_eigenvalue(item: object, field: str) -> DiscreteEigenvalue:
    """Parse a ``{"re": .., "im": .., "modulus": ..}`` object (discrete DMD mode)."""
    if not isinstance(item, Mapping):
        raise AnalysisError(f"CLI JSON field {field!r} must be an object with 're'/'im'/'modulus'")
    return DiscreteEigenvalue(
        re=_as_float(item.get("re"), f"{field}.re"),
        im=_as_float(item.get("im"), f"{field}.im"),
        modulus=_as_float(item.get("modulus"), f"{field}.modulus"),
    )


def _parse_koopman(data: Mapping[str, object]) -> KoopmanReport:
    """Parse the ``koopman --json`` object (see ``koopman.rs::render_json``)."""
    singular_raw = _require(data, "singular_values", "koopman")
    if not isinstance(singular_raw, list):
        raise AnalysisError("`lawsynth koopman` JSON 'singular_values' must be a list")
    discrete_raw = _require(data, "discrete_eigenvalues", "koopman")
    if not isinstance(discrete_raw, list):
        raise AnalysisError("`lawsynth koopman` JSON 'discrete_eigenvalues' must be a list")
    continuous_raw = _require(data, "continuous_eigenvalues", "koopman")
    if not isinstance(continuous_raw, list):
        raise AnalysisError("`lawsynth koopman` JSON 'continuous_eigenvalues' must be a list")
    return KoopmanReport(
        source=str(data.get("source", "")),
        states=tuple(str(state) for state in _require(data, "states", "koopman")),  # type: ignore[union-attr]
        rank=_as_int(_require(data, "rank", "koopman"), "rank"),
        dt=_as_float(_require(data, "dt", "koopman"), "dt"),
        singular_values=tuple(_as_float(value, "singular_values[]") for value in singular_raw),
        discrete_eigenvalues=tuple(
            _parse_discrete_eigenvalue(item, "discrete_eigenvalues[]") for item in discrete_raw
        ),
        continuous_eigenvalues=tuple(
            _parse_eigenvalue(item, "continuous_eigenvalues[]") for item in continuous_raw
        ),
        spectral_radius=_as_float(_require(data, "spectral_radius", "koopman"), "spectral_radius"),
        stable=bool(_require(data, "stable", "koopman")),
    )


def _parse_sde_law(data: object, field: str) -> SdeLaw:
    """Parse a drift/diffusion law object (see ``sde.rs::law_json``)."""
    if not isinstance(data, Mapping):
        raise AnalysisError(f"`lawsynth sde` JSON {field!r} must be an object")
    terms_raw = _require(data, "terms", "sde")
    if not isinstance(terms_raw, list):
        raise AnalysisError(f"`lawsynth sde` JSON {field}.terms must be a list")
    terms = tuple(
        SdeTerm(
            label=str(_require(term, "label", "sde")),
            power=_as_int(_require(term, "power", "sde"), f"{field}.terms[].power"),
            coefficient=_as_float(
                _require(term, "coefficient", "sde"), f"{field}.terms[].coefficient"
            ),
        )
        for term in terms_raw
    )
    return SdeLaw(
        terms=terms,
        residual_sum_squares=_as_float(
            _require(data, "residual_sum_squares", "sde"), f"{field}.residual_sum_squares"
        ),
    )


def _parse_sde(data: Mapping[str, object]) -> SdeReport:
    """Parse the ``sde --json`` object (see ``sde.rs::render_json``)."""
    states_raw = _require(data, "states", "sde")
    if not isinstance(states_raw, list):
        raise AnalysisError("`lawsynth sde` JSON 'states' must be a list")
    states = []
    for entry in states_raw:
        if not isinstance(entry, Mapping):
            raise AnalysisError("`lawsynth sde` JSON 'states[]' must be an object")
        bins_raw = _require(entry, "bins", "sde")
        if not isinstance(bins_raw, list):
            raise AnalysisError("`lawsynth sde` JSON 'states[].bins' must be a list")
        bins = tuple(
            SdeBin(
                x_center=_as_float(_require(bin_, "x_center", "sde"), "bins[].x_center"),
                drift=_as_float(_require(bin_, "drift", "sde"), "bins[].drift"),
                diffusion=_as_float(_require(bin_, "diffusion", "sde"), "bins[].diffusion"),
                count=_as_int(_require(bin_, "count", "sde"), "bins[].count"),
            )
            for bin_ in bins_raw
        )
        states.append(
            SdeState(
                state=str(_require(entry, "state", "sde")),
                trusted_bins=_as_int(_require(entry, "trusted_bins", "sde"), "trusted_bins"),
                drift=_parse_sde_law(_require(entry, "drift", "sde"), "drift"),
                diffusion=_parse_sde_law(_require(entry, "diffusion", "sde"), "diffusion"),
                bins=bins,
            )
        )
    return SdeReport(
        source=str(data.get("source", "")),
        dt=_as_float(_require(data, "dt", "sde"), "dt"),
        increments=_as_int(_require(data, "increments", "sde"), "increments"),
        states=tuple(states),
    )


def _parse_pde(data: Mapping[str, object]) -> PdeReport:
    """Parse the ``pde --json`` object (see ``pde.rs::render_json``)."""
    terms_raw = _require(data, "terms", "pde")
    if not isinstance(terms_raw, list):
        raise AnalysisError("`lawsynth pde` JSON 'terms' must be a list")
    terms = tuple(
        PdeTerm(
            label=str(_require(term, "label", "pde")),
            u_power=_as_int(_require(term, "u_power", "pde"), "terms[].u_power"),
            derivative_order=_as_int(
                _require(term, "derivative_order", "pde"), "terms[].derivative_order"
            ),
            coefficient=_as_float(_require(term, "coefficient", "pde"), "terms[].coefficient"),
        )
        for term in terms_raw
    )
    return PdeReport(
        source=str(data.get("source", "")),
        variable=str(_require(data, "variable", "pde")),
        time_snapshots=_as_int(_require(data, "time_snapshots", "pde"), "time_snapshots"),
        spatial_points=_as_int(_require(data, "spatial_points", "pde"), "spatial_points"),
        dx=_as_float(_require(data, "dx", "pde"), "dx"),
        dt=_as_float(_require(data, "dt", "pde"), "dt"),
        interior_points=_as_int(_require(data, "interior_points", "pde"), "interior_points"),
        residual_sum_squares=_as_float(
            _require(data, "residual_sum_squares", "pde"), "residual_sum_squares"
        ),
        law=str(_require(data, "law", "pde")),
        terms=terms,
    )


def _parse_invariants(data: Mapping[str, object]) -> InvariantReport:
    """Parse the ``invariants --json`` object (see ``invariants.rs::render_json``)."""
    box_raw = _require(data, "sample_box", "invariants")
    if not isinstance(box_raw, Mapping):
        raise AnalysisError("`lawsynth invariants` JSON 'sample_box' must be an object")
    invariants_raw = _require(data, "invariants", "invariants")
    if not isinstance(invariants_raw, list):
        raise AnalysisError("`lawsynth invariants` JSON 'invariants' must be a list")
    labels_raw = _require(data, "basis_labels", "invariants")
    if not isinstance(labels_raw, list):
        raise AnalysisError("`lawsynth invariants` JSON 'basis_labels' must be a list")
    invariants = tuple(
        Invariant(
            combination=str(_require(entry, "combination", "invariants")),
            coefficients=tuple(
                _as_float(value, "invariants[].coefficients[]")
                for value in entry.get("coefficients", [])
            ),
            residual=_as_float(
                _require(entry, "residual", "invariants"), "invariants[].residual"
            ),
            singular_value=_as_float(
                _require(entry, "singular_value", "invariants"), "invariants[].singular_value"
            ),
        )
        for entry in invariants_raw
    )
    return InvariantReport(
        world=str(data.get("world", "")),
        degree=_as_int(_require(data, "degree", "invariants"), "degree"),
        trigonometric=bool(_require(data, "trigonometric", "invariants")),
        sample_box=(
            _as_float(_require(box_raw, "lo", "invariants"), "sample_box.lo"),
            _as_float(_require(box_raw, "hi", "invariants"), "sample_box.hi"),
        ),
        resolution=_as_int(_require(data, "resolution", "invariants"), "resolution"),
        tolerance=_as_float(_require(data, "tolerance", "invariants"), "tolerance"),
        basis_labels=tuple(str(label) for label in labels_raw),
        invariants=invariants,
    )


def _parse_analyze_section(data: object, kind: str, parser):
    """Dispatch one ``analyze`` sub-object: a skip marker or the standalone shape.

    A ``{"skipped": true, "note": ...}`` sub-object becomes a small :class:`Skipped`
    marker; anything else is handed to ``parser`` — the exact standalone
    ``_parse_stability`` / ``_parse_lyapunov`` / ``_parse_invariants`` — so the
    composed sub-report is the same type that command's parser already returns.
    """
    if not isinstance(data, Mapping):
        raise AnalysisError(f"`lawsynth analyze` JSON {kind!r} sub-object must be an object")
    if data.get("skipped") is True:
        return Skipped(note=str(data.get("note", "")))
    return parser(data)


def _parse_analyze(data: Mapping[str, object]) -> AnalyzeReport:
    """Parse the ``analyze --json`` object (see ``analyze.rs::render_json``).

    Composes the existing standalone parsers over the sub-objects via
    :func:`_parse_analyze_section`: each of ``stability`` / ``lyapunov`` /
    ``invariants`` is either a :class:`Skipped` marker or, reusing
    ``_parse_stability`` / ``_parse_lyapunov`` / ``_parse_invariants`` unchanged, a
    :class:`StabilityReport` / :class:`LyapunovReport` / :class:`InvariantReport`.
    """
    return AnalyzeReport(
        world=str(data.get("world", "")),
        states=tuple(str(state) for state in _require(data, "states", "analyze")),  # type: ignore[union-attr]
        stability=_parse_analyze_section(
            _require(data, "stability", "analyze"), "stability", _parse_stability
        ),
        lyapunov=_parse_analyze_section(
            _require(data, "lyapunov", "analyze"), "lyapunov", _parse_lyapunov
        ),
        invariants=_parse_analyze_section(
            _require(data, "invariants", "analyze"), "invariants", _parse_invariants
        ),
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


def _format_range(value: str | tuple[float, float] | Sequence[float]) -> str:
    """Format a sweep range as ``MIN:MAX`` (pass-through for strings)."""
    if isinstance(value, str):
        text = value.strip()
        if not text:
            raise ValidationError("range must not be empty")
        return text
    low, high = value  # type: ignore[misc]
    low, high = float(low), float(high)
    if low > high:
        raise ValidationError(f"range ({low}, {high}) has minimum above maximum")
    return f"{low}:{high}"


def _format_identifiers(names: Sequence[str], *, label: str) -> str:
    """Join a non-empty sequence of identifier names as ``A,B,C``."""
    parts = [str(name).strip() for name in names]
    if not parts or any(not part for part in parts):
        raise ValidationError(f"{label} must contain at least one non-empty name")
    return ",".join(parts)


def _format_poles(poles: Sequence[object]) -> str:
    """Format desired poles as ``RE[:IM][,RE[:IM]...]``.

    Accepts real floats (``-2.0``), ``(re, im)`` pairs for complex poles, or raw
    ``"RE"``/``"RE:IM"`` strings passed straight through.
    """
    parts: list[str] = []
    for pole in poles:
        if isinstance(pole, str):
            parts.append(pole.strip())
        elif isinstance(pole, (tuple, list)):
            real, imag = pole
            parts.append(f"{float(real)}:{float(imag)}")
        else:
            parts.append(f"{float(pole)}")  # type: ignore[arg-type]
    if not parts:
        raise ValidationError("poles must contain at least one pole")
    return ",".join(parts)


def _initial_args(
    initial: Mapping[str, float] | Sequence[tuple[str, float]] | None,
) -> list[str]:
    """Build repeated ``--initial NAME=VALUE`` flags from a mapping or pairs."""
    if initial is None:
        return []
    items = initial.items() if isinstance(initial, Mapping) else initial
    args: list[str] = []
    for name, value in items:
        args += ["--initial", f"{str(name)}={float(value)}"]
    return args


def _format_assignments(
    assignments: Mapping[str, float] | Sequence[tuple[str, float]],
    *,
    label: str,
) -> str:
    """Format state assignments as a single ``NAME=VALUE[,NAME=VALUE...]`` value.

    The ``lyapunov`` and ``mpc`` commands take one comma-separated flag value (via
    the engine's ``parse_state_vector``, which requires exactly one entry per state
    of the world), unlike ``sensitivity``'s repeated ``--initial`` flags.
    """
    items = list(assignments.items() if isinstance(assignments, Mapping) else assignments)
    if not items:
        raise ValidationError(f"{label} must assign at least one state")
    parts: list[str] = []
    for name, value in items:
        text = str(name).strip()
        if not text:
            raise ValidationError(f"{label} contains an empty state name")
        parts.append(f"{text}={float(value)}")
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


def bifurcation(
    world_path: str | PathLike[str],
    *,
    parameter: str,
    range: str | tuple[float, float] | Sequence[float],  # noqa: A002 - mirrors --range
    box: str | Sequence[tuple[float, float]],
    steps: int | None = None,
    grid: int | None = None,
) -> BifurcationReport:
    """Continue a free parameter and detect bifurcations via the CLI engine.

    Runs ``lawsynth bifurcation WORLD --parameter NAME --range MIN:MAX --box ...
    --json`` and parses the result into a :class:`BifurcationReport`. ``parameter``
    names the swept symbol (it **must appear in at least one law**: a discovered
    world that inlines its coefficients as constants has no free parameter to
    sweep, and the CLI rejects it with a :class:`CliError`). ``range`` is the
    ``MIN:MAX`` sweep interval (string or ``(min, max)``); ``box`` is the per-state
    search box (as in :func:`stability`); ``steps`` sets the number of parameter
    grid points and ``grid`` the fixed-point search resolution. Every other
    declared parameter is pinned at its stored value.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` if the command fails (unreadable world, un-parameterized
    field, ...).
    """
    param = str(parameter).strip()
    if not param:
        raise ValidationError("parameter must be a non-empty name")
    args: list[str] = [
        "bifurcation", str(world_path),
        "--parameter", param,
        "--range", _format_range(range),
        "--box", _format_box(box),
    ]
    if steps is not None:
        args += ["--steps", str(int(steps))]
    if grid is not None:
        args += ["--grid", str(int(grid))]
    args.append("--json")
    return _parse_bifurcation(_run_cli(args))


def sensitivity(
    world_path: str | PathLike[str],
    *,
    parameters: Sequence[str],
    initial: Mapping[str, float] | Sequence[tuple[str, float]] | None = None,
    start: float | None = None,
    dt: float | None = None,
    steps: int | None = None,
) -> SensitivityReport:
    """Compute forward sensitivities ``∂x_i/∂θ_j`` at the final time via the CLI engine.

    Runs ``lawsynth sensitivity WORLD --parameters ... --json`` and parses the
    result into a :class:`SensitivityReport`. Each name in ``parameters`` must be a
    declared world parameter (its value is read from the world; every other
    declared parameter is pinned). ``initial`` sets initial state components
    (mapping ``{state: value}`` or ``(state, value)`` pairs); unset states default
    to ``0``. ``start`` / ``dt`` / ``steps`` control the integration window.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit (e.g. a name that is not a declared
    parameter).
    """
    args: list[str] = [
        "sensitivity", str(world_path),
        "--parameters", _format_identifiers(parameters, label="parameters"),
    ]
    args += _initial_args(initial)
    if start is not None:
        args += ["--start", repr(float(start))]
    if dt is not None:
        args += ["--dt", repr(float(dt))]
    if steps is not None:
        args += ["--steps", str(int(steps))]
    args.append("--json")
    return _parse_sensitivity(_run_cli(args))


def estimate(
    world_path: str | PathLike[str],
    *,
    box: str | Sequence[tuple[float, float]],
    measure: Sequence[str],
    poles: Sequence[object] | None = None,
    kalman: bool = False,
    process_var: float | None = None,
    measurement_var: float | None = None,
    grid: int | None = None,
) -> EstimateReport:
    """Design a state estimator (observer / Kalman) via the CLI engine.

    Runs ``lawsynth estimate WORLD --box ... --measure ... --json`` and parses the
    result into an :class:`EstimateReport`. The world's field is linearized at the
    first fixed point inside ``box``; ``measure`` names the measured states (the
    output map ``C``). With ``poles`` the error poles of ``A − L C`` are placed by
    Ackermann's formula — this is **single-output** (pass exactly one ``measure``
    state) and needs one pole per state; each pole is a real float, a ``(re, im)``
    pair, or a raw ``"RE"``/``"RE:IM"`` string. With ``kalman=True`` the
    steady-state Kalman gain is designed instead (``process_var`` / ``measurement_var``
    scale ``Q = qI`` / ``R = rI``, default ``1``), accepting several measured
    states. ``poles`` and ``kalman`` are mutually exclusive. ``grid`` sets the
    fixed-point search resolution.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit (no fixed point in the box, a
    single-output constraint violation, ...).
    """
    if kalman and poles is not None:
        raise ValidationError("choose either poles (pole placement) or kalman, not both")
    args: list[str] = [
        "estimate", str(world_path),
        "--box", _format_box(box),
        "--measure", _format_identifiers(measure, label="measure"),
    ]
    if kalman:
        args.append("--kalman")
        if process_var is not None:
            args += ["--process-var", repr(float(process_var))]
        if measurement_var is not None:
            args += ["--measurement-var", repr(float(measurement_var))]
    else:
        if poles is None:
            raise ValidationError("pole placement needs poles=..., or pass kalman=True")
        args += ["--poles", _format_poles(poles)]
    if grid is not None:
        args += ["--grid", str(int(grid))]
    args.append("--json")
    return _parse_estimate(_run_cli(args))


def reduce(
    world_path: str | PathLike[str],
    *,
    box: str | Sequence[tuple[float, float]],
    order: int | None = None,
    tolerance: float | None = None,
    measure: Sequence[str] | None = None,
    grid: int | None = None,
) -> ReductionReport:
    """Reduce the linearized model by balanced truncation via the CLI engine.

    Runs ``lawsynth reduce WORLD --box ... --json`` and parses the result into a
    :class:`ReductionReport`. The world's field is linearized at the first fixed
    point inside ``box``; balanced truncation additionally requires that fixed
    point to be **stable (Hurwitz)** — an unstable equilibrium is rejected by the
    engine with a :class:`CliError`. Choose the reduced order with ``order`` (keep
    ``K`` states) **or** ``tolerance`` (keep the fewest states whose discarded
    Hankel energy fraction is at most ``T``); exactly one is required. ``measure``
    selects the output states for ``C`` (default ``C = I``); ``grid`` sets the
    fixed-point search resolution.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit (no fixed point, an unstable fixed point,
    ...).
    """
    if order is not None and tolerance is not None:
        raise ValidationError("choose either order=K or tolerance=T, not both")
    if order is None and tolerance is None:
        raise ValidationError("one of order=K or tolerance=T is required")
    args: list[str] = ["reduce", str(world_path), "--box", _format_box(box)]
    if order is not None:
        args += ["--order", str(int(order))]
    if tolerance is not None:
        args += ["--tolerance", repr(float(tolerance))]
    if measure is not None:
        args += ["--measure", _format_identifiers(measure, label="measure")]
    if grid is not None:
        args += ["--grid", str(int(grid))]
    args.append("--json")
    return _parse_reduce(_run_cli(args))


def lyapunov(
    world_path: str | PathLike[str],
    *,
    initial: Mapping[str, float] | Sequence[tuple[str, float]],
    dt: float | None = None,
    steps: int | None = None,
    reorth: int | None = None,
    transient: float | None = None,
) -> LyapunovReport:
    """Estimate the Lyapunov spectrum (chaos diagnostic) of a world via the CLI engine.

    Runs ``lawsynth lyapunov WORLD --initial ... --json`` and parses the result
    into a :class:`LyapunovReport`. ``initial`` is the launch point (mapping
    ``{state: value}`` or ``(state, value)`` pairs) and must assign **every** state
    of the world exactly once — the engine folds it into a single comma-separated
    ``--initial`` flag. ``dt`` is the integration step, ``steps`` the number of
    steps, ``reorth`` the QR reorthonormalization interval, and ``transient`` the
    fraction (in ``[0, 1)``) discarded before averaging.

    The spectrum is a **time-averaged estimate**, not exact: its accuracy grows
    with ``steps`` and shrinks with ``dt``, the initial condition should sit in the
    target attractor's basin, and the exponent ``sum`` is the tightest quantity.
    ``report.chaotic`` is exactly ``report.largest > 0``.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit (unreadable world, an ``--initial`` that
    does not cover the states, ...).
    """
    args: list[str] = [
        "lyapunov", str(world_path),
        "--initial", _format_assignments(initial, label="initial"),
    ]
    if dt is not None:
        args += ["--dt", repr(float(dt))]
    if steps is not None:
        args += ["--steps", str(int(steps))]
    if reorth is not None:
        args += ["--reorth", str(int(reorth))]
    if transient is not None:
        args += ["--transient", repr(float(transient))]
    args.append("--json")
    return _parse_lyapunov(_run_cli(args))


def basins(
    world_path: str | PathLike[str],
    *,
    box: str | Sequence[tuple[float, float]],
    resolution: int | None = None,
    dt: float | None = None,
    max_time: float | None = None,
    tolerance: float | None = None,
) -> BasinReport:
    """Map the basins of attraction of a multistable world via the CLI engine.

    Runs ``lawsynth basins WORLD --box ... --json`` and parses the result into a
    :class:`BasinReport`. ``box`` is the required search box (one ``(low, high)``
    per state, or the raw ``"LOW:HIGH,LOW:HIGH"`` string): it fixes both the
    initial-condition grid and the escape region. ``resolution`` sets the grid
    samples per axis, ``dt`` the RK4 step, ``max_time`` the settle horizon, and
    ``tolerance`` the attractor-convergence radius.

    **Only fixed-point attractors are recognized**: a limit cycle or strange
    attractor reads as ``undetermined`` rather than being forced into a basin, and
    a trajectory leaving the box is ``escaped``. An empty ``attractors`` means no
    stable fixed point was found inside the box — widen it.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit.
    """
    args: list[str] = ["basins", str(world_path), "--box", _format_box(box)]
    if resolution is not None:
        args += ["--resolution", str(int(resolution))]
    if dt is not None:
        args += ["--dt", repr(float(dt))]
    if max_time is not None:
        args += ["--max-time", repr(float(max_time))]
    if tolerance is not None:
        args += ["--tolerance", repr(float(tolerance))]
    args.append("--json")
    return _parse_basins(_run_cli(args))


def network(
    dataset_path: str | PathLike[str],
    *,
    states: Sequence[str],
    degree: int | None = None,
    threshold: float | None = None,
    edge_threshold: float | None = None,
    time: str = "time",
) -> NetworkModel:
    """Discover the directed coupling graph of a networked system via the CLI engine.

    Runs ``lawsynth network OBSERVATIONS --state ... --json`` and parses the result
    into a :class:`NetworkModel`. Each name in ``states`` is a network node (a
    dataset column); its derivative is sparsely regressed onto a shared polynomial
    library over all nodes, and a surviving cross term ``x_driver`` in node
    ``target``'s equation becomes a directed edge ``driver -> target``. **At least
    two nodes are required.** ``degree`` sets the library degree (``1`` = linear
    couplings), ``threshold`` the per-term sparsity cutoff, ``edge_threshold`` the
    minimum aggregated strength for an edge, and ``time`` the time column
    (default ``"time"``).

    The recovered graph is **correlational, not causal**: a confounder or common
    drive can induce a spurious edge, and heavy noise degrades recovery as it does
    for strong-form SINDy.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit (a named state that is not a column, ...).
    """
    node_list = [str(name) for name in states]
    if len(node_list) < 2:
        raise ValidationError("network requires at least two node names in states")
    args: list[str] = [
        "network", str(dataset_path),
        "--state", _format_identifiers(node_list, label="states"),
        "--time", str(time),
    ]
    if degree is not None:
        args += ["--degree", str(int(degree))]
    if threshold is not None:
        args += ["--threshold", repr(float(threshold))]
    if edge_threshold is not None:
        args += ["--edge-threshold", repr(float(edge_threshold))]
    args.append("--json")
    return _parse_network(_run_cli(args))


def mpc(
    world_path: str | PathLike[str],
    *,
    control: Sequence[str],
    setpoint: Mapping[str, float] | Sequence[tuple[str, float]],
    initial: Mapping[str, float] | Sequence[tuple[str, float]],
    dt: float | None = None,
    steps: int | None = None,
    q: float | None = None,
    r: float | None = None,
    u_min: float | None = None,
    u_max: float | None = None,
) -> MpcResult:
    """Regulate a forced world to a setpoint by LQR-MPC via the CLI engine.

    Runs ``lawsynth mpc WORLD --control ... --setpoint ... --initial ... --json``
    and parses the result into an :class:`MpcResult`. ``control`` names the world's
    control symbols (the forcing channels of ``ẋ = f(x, u)``). ``setpoint`` and
    ``initial`` are state assignments (mapping ``{state: value}`` or
    ``(state, value)`` pairs) and must each assign **every** state exactly once —
    the engine folds each into a single comma-separated flag. ``dt`` is the step,
    ``steps`` the closed-loop horizon, ``q`` / ``r`` scale the identity state /
    control weights ``Q = q·I`` / ``R = r·I``. ``u_min`` / ``u_max`` saturate every
    control channel and must be given **together** (two-sided) or not at all.

    This is **successive-linearization LQR-MPC**, not a constrained QP-MPC: the
    local LQR needs a **stabilizable** linearization, saturation is a clamp (not a
    constraint-optimal projection), optimality is only local, and there is no
    horizon/feasibility guarantee. A failed LQR design (unstabilizable
    linearization, non-positive-definite ``R``) surfaces as a :class:`CliError`.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit.
    """
    control_list = [str(name) for name in control]
    if not control_list:
        raise ValidationError("mpc requires at least one control name")
    if u_min is None and u_max is not None or u_min is not None and u_max is None:
        raise ValidationError("provide both u_min and u_max (two-sided saturation) or neither")
    args: list[str] = [
        "mpc", str(world_path),
        "--control", _format_identifiers(control_list, label="control"),
        "--setpoint", _format_assignments(setpoint, label="setpoint"),
        "--initial", _format_assignments(initial, label="initial"),
    ]
    if dt is not None:
        args += ["--dt", repr(float(dt))]
    if steps is not None:
        args += ["--steps", str(int(steps))]
    if q is not None:
        args += ["--q", repr(float(q))]
    if r is not None:
        args += ["--r", repr(float(r))]
    if u_min is not None:
        args += ["--u-min", repr(float(u_min))]
    if u_max is not None:
        args += ["--u-max", repr(float(u_max))]
    args.append("--json")
    return _parse_mpc(_run_cli(args))


def koopman(
    dataset_path: str | PathLike[str],
    *,
    states: Sequence[str],
    time: str = "time",
    rank: int | None = None,
) -> KoopmanReport:
    """Discover the best-fit linear operator ``x' ≈ A x`` (Koopman/DMD) via the CLI engine.

    Runs ``lawsynth koopman DATASET --state ... --time ... --json`` and parses the
    result into a :class:`KoopmanReport`. Dynamic Mode Decomposition fits the
    best-fit **linear operator** ``A`` over the dataset's snapshot pairs and
    reports its spectrum — the discrete-time eigenvalues ``λ`` (per step, with
    modulus ``|λ|``), the continuous-time eigenvalues ``μ = ln(λ)/Δt``, the
    singular values, and a spectral-radius stability verdict. ``states`` names the
    observable columns; ``time`` is the time column (default ``"time"``); ``rank``
    truncates the SVD (default: full usable rank = ``min(states, snapshot pairs)``).

    The recovered object is honestly a **linear (or lifted-linear) approximation**
    of the dynamics, not a symbolic nonlinear world — unlike :func:`discover`, DMD
    writes no ``.lsworld`` bundle. (Weak-form symbolic discovery is a *different*
    method, reached via the CLI's ``discover --method weak-form``.)

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit (a named state that is not a column, a
    ``--rank`` above the usable maximum, a non-increasing time axis, ...).
    """
    args: list[str] = [
        "koopman", str(dataset_path),
        "--state", _format_identifiers(states, label="states"),
        "--time", str(time),
    ]
    if rank is not None:
        args += ["--rank", str(int(rank))]
    args.append("--json")
    return _parse_koopman(_run_cli(args))


def sde(
    dataset_path: str | PathLike[str],
    *,
    states: Sequence[str],
    time: str = "time",
    bins: int | None = None,
    min_bin: int | None = None,
    degree: int | None = None,
    threshold: float | None = None,
) -> SdeReport:
    """Discover a stochastic (drift/diffusion) law from one sample path via the CLI engine.

    Runs ``lawsynth sde OBSERVATIONS --state ... --time ... --json`` and parses the
    result into an :class:`SdeReport`. For a diagonal-noise Itô SDE ``dX = a(X) dt
    + b(X) dW``, the drift ``a(x)`` and diffusion ``b²(x)`` are estimated per state
    from binned Kramers–Moyal conditional moments and sparse-regressed onto a
    polynomial library. ``states`` names the observed state columns; ``time`` is
    the time column (default ``"time"``); ``bins`` sets the state-space partition
    count, ``min_bin`` the minimum occupancy a bin needs to be trusted, ``degree``
    the library degree, and ``threshold`` the sparsity cutoff.

    This is a **statistical estimator**, not an exact identity: accuracy grows with
    path length and rarely-visited (low-occupancy) bins are unreliable; it writes
    no ``.lsworld`` bundle.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit (a named state that is not a column, ...).
    """
    args: list[str] = [
        "sde", str(dataset_path),
        "--state", _format_identifiers(states, label="states"),
        "--time", str(time),
    ]
    if bins is not None:
        args += ["--bins", str(int(bins))]
    if min_bin is not None:
        args += ["--min-bin", str(int(min_bin))]
    if degree is not None:
        args += ["--degree", str(int(degree))]
    if threshold is not None:
        args += ["--threshold", repr(float(threshold))]
    args.append("--json")
    return _parse_sde(_run_cli(args))


def pde(
    field_path: str | PathLike[str],
    *,
    dx: float,
    dt: float,
    degree: int | None = None,
    order: int | None = None,
    threshold: float | None = None,
) -> PdeReport:
    """Discover a 1-D evolution PDE ``u_t = F(u, u_x, u_xx, …)`` (PDE-FIND) via the CLI engine.

    Runs ``lawsynth pde FIELD --dx DX --dt DT --json`` and parses the result into a
    :class:`PdeReport`. Spatial and temporal derivatives are estimated with central
    **finite differences** and the flattened ``u_t`` is sparse-regressed onto a
    differential-term library. ``FIELD`` is a plain rectangular numeric grid **with
    no header** — rows are time snapshots, columns are spatial points. ``dx`` /
    ``dt`` are the (uniform, positive) grid steps; ``degree`` is the maximum field
    power (``u, u², …``), ``order`` the maximum spatial-derivative order
    (``u_x, u_xx, u_xxx``), and ``threshold`` the relative sparsity cutoff.

    Finite differencing amplifies noise, so recovery is **noise-sensitive and needs
    a resolved grid** — finer grids tighten the coefficients; it writes no
    ``.lsworld`` bundle.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit (a ragged / non-numeric grid, a
    non-positive ``--dx``/``--dt``, ...).
    """
    if float(dx) <= 0.0:
        raise ValidationError("dx must be a positive grid step")
    if float(dt) <= 0.0:
        raise ValidationError("dt must be a positive grid step")
    args: list[str] = [
        "pde", str(field_path),
        "--dx", repr(float(dx)),
        "--dt", repr(float(dt)),
    ]
    if degree is not None:
        args += ["--degree", str(int(degree))]
    if order is not None:
        args += ["--order", str(int(order))]
    if threshold is not None:
        args += ["--threshold", repr(float(threshold))]
    args.append("--json")
    return _parse_pde(_run_cli(args))


def invariants(
    world_path: str | PathLike[str],
    *,
    box: str | tuple[float, float] | Sequence[float] | None = None,
    degree: int | None = None,
    trig: bool = False,
    resolution: int | None = None,
    tolerance: float | None = None,
) -> InvariantReport:
    """Search for conserved quantities ``H(x)`` of a world via the CLI engine.

    Runs ``lawsynth invariants WORLD --json`` and parses the result into an
    :class:`InvariantReport`. The world's laws are read as an autonomous field
    ``ẋ = f(x)`` (every declared parameter pinned) and each nonconstant combination
    of a candidate library whose Lie derivative ``L_f H = ∇H·f`` vanishes on the
    sample grid is reported as an :class:`Invariant`. ``box`` is the **single shared
    per-axis interval** ``[LO, HI]^n`` the grid samples — the CLI's ``--box`` here
    takes one ``LOW:HIGH`` (unlike the per-state box of :func:`stability` /
    :func:`analyze`), so pass a ``(lo, hi)`` pair or the raw ``"LO:HI"`` string.
    ``degree`` sets the monomial library degree, ``trig=True`` adds ``sin``/``cos``
    terms, ``resolution`` the grid points per axis, and ``tolerance`` the near-null
    cutoff on the Lie-derivative singular values.

    The search is **library-bounded**: an empty ``report.invariants`` means no
    conserved quantity expressible in the chosen library was found within tolerance,
    not a proof that none exists — raise ``degree``, add ``trig``, or widen ``box``.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit (unreadable world, a decomposition that
    fails to converge on the chosen box/resolution, ...).
    """
    args: list[str] = ["invariants", str(world_path)]
    if box is not None:
        args += ["--box", _format_range(box)]
    if degree is not None:
        args += ["--degree", str(int(degree))]
    if trig:
        args.append("--trig")
    if resolution is not None:
        args += ["--resolution", str(int(resolution))]
    if tolerance is not None:
        args += ["--tolerance", repr(float(tolerance))]
    args.append("--json")
    return _parse_invariants(_run_cli(args))


def analyze(
    world_path: str | PathLike[str],
    *,
    box: str | Sequence[tuple[float, float]],
    grid: int | None = None,
    initial: Mapping[str, float] | Sequence[tuple[str, float]] | None = None,
    dt: float | None = None,
    steps: int | None = None,
    degree: int | None = None,
    trig: bool = False,
) -> AnalyzeReport:
    """Run the combined one-shot dynamics analysis of a world via the CLI engine.

    Runs ``lawsynth analyze WORLD --box ... --json`` and parses the result into an
    :class:`AnalyzeReport`. In one pass the engine runs :func:`stability` (fixed
    points inside the required ``box`` and their linear classification),
    :func:`lyapunov` (the spectrum from ``initial``, defaulting to the origin), and
    :func:`invariants` (conserved quantities over a degree-``D`` library). ``box``
    is the mandatory **per-state** search box (one ``(low, high)`` per state, in
    state order, or the raw ``"LOW:HIGH,LOW:HIGH"`` string) that drives the
    stability search. ``grid`` sets the fixed-point search resolution; ``initial``
    the Lyapunov launch point (mapping ``{state: value}`` or ``(state, value)``
    pairs, assigning **every** state exactly once — folded into a single
    ``--initial`` flag); ``dt`` / ``steps`` the Lyapunov integration; ``degree`` and
    ``trig`` the invariant library.

    Each sub-object mirrors the standalone command's ``--json`` exactly, so
    ``report.stability`` / ``report.lyapunov`` / ``report.invariants`` are the very
    :class:`StabilityReport` / :class:`LyapunovReport` / :class:`InvariantReport`
    those commands return. A part that **cannot run for this world** (a
    non-autonomous field, a Lyapunov run that diverges) is a :class:`Skipped` marker
    rather than a failure of the whole command — only an unloadable world or a
    missing ``box`` is a hard :class:`CliError` / :class:`ValidationError`.

    Raises :class:`MissingBinaryError` if the CLI is not built and
    :class:`CliError` on a non-zero exit.
    """
    args: list[str] = ["analyze", str(world_path), "--box", _format_box(box)]
    if grid is not None:
        args += ["--grid", str(int(grid))]
    if initial is not None:
        args += ["--initial", _format_assignments(initial, label="initial")]
    if dt is not None:
        args += ["--dt", repr(float(dt))]
    if steps is not None:
        args += ["--steps", str(int(steps))]
    if degree is not None:
        args += ["--degree", str(int(degree))]
    if trig:
        args.append("--trig")
    args.append("--json")
    return _parse_analyze(_run_cli(args))


# --------------------------------------------------------------------------- #
# Attach convenience methods to DiscoveryResult / Study (best-effort, lazy)     #
# --------------------------------------------------------------------------- #


def _install() -> None:
    """Attach world-analysis convenience methods to Study / DiscoveryResult.

    Each of the CLI's world analyses reads a ``.lsworld`` bundle, so the
    convenience methods save the in-memory discovered world to a temporary bundle
    and analyse that (mirrors export/tracking). Note that ``bifurcation`` and
    ``sensitivity`` require *declared parameters* in the laws — a discovered world
    that inlines its coefficients as constants exposes none, and the CLI will say
    so honestly via :class:`CliError`. ``reduce`` requires a stable (Hurwitz)
    fixed point. The methods are thin pass-throughs; the engine's honest errors
    surface unchanged.
    """
    try:
        from .study import DiscoveryResult, Study
    except Exception:  # pragma: no cover - defensive at import time
        return

    def _make_method(function):
        def _method(self: object, **kwargs: object):
            import tempfile

            with tempfile.TemporaryDirectory(prefix="lawsynth-analysis-") as tmp:
                target = Path(tmp) / "world.lsworld"
                self.save(target)  # type: ignore[attr-defined]
                return function(target, **kwargs)

        _method.__name__ = function.__name__
        _method.__qualname__ = function.__name__
        _method.__doc__ = (
            f"Convenience wrapper: save this world to a temporary bundle and call "
            f"``lawsynth.analysis.{function.__name__}`` on it. See that function for "
            f"parameters and honest preconditions."
        )
        return _method

    methods = {
        "stability": stability,
        "bifurcation": bifurcation,
        "sensitivity": sensitivity,
        "estimate": estimate,
        "reduce": reduce,
        "lyapunov": lyapunov,
        "basins": basins,
        "mpc": mpc,
        "invariants": invariants,
        "analyze": analyze,
    }
    for cls in (Study, DiscoveryResult):
        for name, function in methods.items():
            if not hasattr(cls, name):
                setattr(cls, name, _make_method(function))


_install()
