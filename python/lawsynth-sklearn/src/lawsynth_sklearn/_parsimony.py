"""Auto-parsimony: a deterministic Cov/Var complexity price over a Pareto sweep.

``parsimony='auto'`` runs discovery across a deterministic grid of sparsity
thresholds, scores each resulting world by ``(complexity, loss)``, filters to the
Pareto front, and derives a complexity price

    λ = Cov(complexity, loss) / Var(complexity)

over the front (the marginal rate at which added terms buy lower loss — the same
heuristic gplearn uses for ``parsimony_coefficient='auto'``). The model that
minimises the penalised objective ``loss + |λ|·complexity`` is selected. Every
step is deterministic and offline, so the chosen threshold and λ reproduce
exactly.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Callable

from lawsynth.candidate import CandidateMetrics
from lawsynth.frontier import pareto_front


@dataclass(frozen=True)
class ParsimonyCandidate:
    threshold: float
    complexity: int
    loss: float
    on_front: bool
    penalized: float


@dataclass(frozen=True)
class ParsimonyResult:
    """Outcome of an auto-parsimony sweep."""

    parsimony_coefficient: float  # signed Cov/Var slope over the Pareto front
    threshold: float  # the selected sparsity threshold
    candidates: tuple[ParsimonyCandidate, ...]

    @property
    def selected(self) -> ParsimonyCandidate:
        return min(
            (c for c in self.candidates if c.threshold == self.threshold),
            key=lambda c: c.penalized,
        )


def default_threshold_grid(base_threshold: float) -> tuple[float, ...]:
    """A deterministic multiplicative grid of thresholds around ``base``."""
    base = base_threshold if base_threshold > 0 else 0.05
    factors = (0.25, 0.5, 1.0, 2.0, 4.0, 8.0)
    grid = sorted({round(base * factor, 12) for factor in factors if base * factor > 0})
    return tuple(grid)


def candidate_metrics(result: object) -> tuple[int, float]:
    """Return ``(complexity, loss)`` for a discovered world.

    Complexity is the total number of retained terms across all laws; loss is
    ``1 − mean(R²)`` over the modelled states, clamped at zero.
    """
    explanation = result.explain()  # type: ignore[attr-defined]
    complexity = sum(len(law.terms) for law in explanation.laws)
    r_squared = [metrics["r_squared"] for metrics in explanation.fit.values()]
    mean_r2 = sum(r_squared) / len(r_squared) if r_squared else 0.0
    loss = max(0.0, 1.0 - mean_r2)
    return complexity, loss


def _cov_over_var(complexity: list[float], loss: list[float]) -> float:
    n = len(complexity)
    if n == 0:
        return 0.0
    mean_c = sum(complexity) / n
    mean_l = sum(loss) / n
    cov = sum((c - mean_c) * (l - mean_l) for c, l in zip(complexity, loss)) / n
    var = sum((c - mean_c) ** 2 for c in complexity) / n
    if var == 0.0:
        return 0.0
    return cov / var


def auto_parsimony(
    discover_fn: Callable[[float], object],
    base_threshold: float,
    *,
    grid: tuple[float, ...] | None = None,
) -> ParsimonyResult:
    """Sweep thresholds, price complexity via Cov/Var, and select a model.

    ``discover_fn(threshold)`` must run discovery at the given sparsity threshold
    and return a ``DiscoveryResult`` (exposing ``.explain()``). The returned
    :class:`ParsimonyResult` carries the signed Cov/Var coefficient, the selected
    threshold, and the full scored Pareto table.
    """
    thresholds = grid if grid is not None else default_threshold_grid(base_threshold)

    scored: list[tuple[float, int, float]] = []
    for threshold in thresholds:
        complexity, loss = candidate_metrics(discover_fn(threshold))
        scored.append((threshold, complexity, loss))

    metrics = tuple(
        CandidateMetrics(mean_squared_error=loss, complexity=complexity)
        for _, complexity, loss in scored
    )
    front_indices = set(pareto_front(metrics))

    front_complexity = [float(scored[i][1]) for i in sorted(front_indices)]
    front_loss = [scored[i][2] for i in sorted(front_indices)]
    slope = _cov_over_var(front_complexity, front_loss)
    price = abs(slope)

    candidates = tuple(
        ParsimonyCandidate(
            threshold=threshold,
            complexity=complexity,
            loss=loss,
            on_front=index in front_indices,
            penalized=loss + price * complexity,
        )
        for index, (threshold, complexity, loss) in enumerate(scored)
    )

    # Select the minimum penalised objective; ties broken toward the sparser
    # (higher-threshold, then lower-complexity) model for reproducibility.
    selected = min(
        candidates,
        key=lambda c: (c.penalized, -c.threshold, c.complexity),
    )
    return ParsimonyResult(
        parsimony_coefficient=slope,
        threshold=selected.threshold,
        candidates=candidates,
    )
