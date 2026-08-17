"""Deterministic feature-library engineering shared by the transformer.

Mirrors the LawSynth discovery feature families — a polynomial library (up to a
total degree) with an optional bias term and optional trigonometric augmentation
(``sin``/``cos`` of each raw feature). A gplearn-style deterministic correlation
prune can drop redundant columns, yielding a least-correlated subset suitable for
downstream linear models.
"""

from __future__ import annotations

from itertools import combinations_with_replacement
from math import cos, sin, sqrt


def polynomial_terms(n_features: int, degree: int, include_bias: bool) -> list[tuple[int, ...]]:
    """Every monomial (as a multiset of feature indices) up to ``degree``.

    A term is a tuple of column indices; ``(0, 0)`` means ``x0**2`` and ``()``
    the bias/constant. Ordering is deterministic and matches
    ``sklearn.preprocessing.PolynomialFeatures`` (ascending total degree).
    """
    if degree < 0:
        raise ValueError("degree must be non-negative")
    terms: list[tuple[int, ...]] = []
    start = 0 if include_bias else 1
    for total in range(start, degree + 1):
        if total == 0:
            terms.append(())
            continue
        terms.extend(combinations_with_replacement(range(n_features), total))
    return terms


def term_name(term: tuple[int, ...], names: list[str]) -> str:
    """Human-readable name for a monomial term (e.g. ``x0^2 x1`` or ``1``)."""
    if not term:
        return "1"
    counts: dict[int, int] = {}
    for index in term:
        counts[index] = counts.get(index, 0) + 1
    parts = []
    for index in sorted(counts):
        power = counts[index]
        parts.append(names[index] if power == 1 else f"{names[index]}^{power}")
    return " ".join(parts)


def build_feature_library(
    rows: list[list[float]],
    names: list[str],
    *,
    degree: int,
    include_bias: bool,
    include_trigonometric: bool,
) -> tuple[list[list[float]], list[str]]:
    """Return ``(feature_rows, feature_names)`` for the requested library."""
    n_features = len(names)
    terms = polynomial_terms(n_features, degree, include_bias)
    feature_names = [term_name(term, names) for term in terms]

    feature_rows: list[list[float]] = []
    for row in rows:
        values = []
        for term in terms:
            product = 1.0
            for index in term:
                product *= row[index]
            values.append(product)
        feature_rows.append(values)

    if include_trigonometric:
        for func, prefix in ((sin, "sin"), (cos, "cos")):
            for index, name in enumerate(names):
                feature_names.append(f"{prefix}({name})")
            for row_index, row in enumerate(rows):
                feature_rows[row_index].extend(func(row[index]) for index in range(n_features))

    return feature_rows, feature_names


def _pearson(a: list[float], b: list[float]) -> float:
    """Pearson correlation; ``0.0`` when either column has no variance."""
    n = len(a)
    if n == 0:
        return 0.0
    mean_a = sum(a) / n
    mean_b = sum(b) / n
    cov = sum((x - mean_a) * (y - mean_b) for x, y in zip(a, b))
    var_a = sum((x - mean_a) ** 2 for x in a)
    var_b = sum((y - mean_b) ** 2 for y in b)
    if var_a == 0.0 or var_b == 0.0:
        return 0.0
    return cov / sqrt(var_a * var_b)


def correlation_prune(
    feature_rows: list[list[float]],
    feature_names: list[str],
    threshold: float,
) -> list[int]:
    """Greedy, deterministic least-correlated subset selection.

    Walks columns left-to-right, keeping a column only when its absolute
    Pearson correlation with every already-kept column stays at or below
    ``threshold``. Deterministic: identical input yields identical kept indices.
    Returns the sorted list of kept column indices.
    """
    if not (0.0 < threshold <= 1.0):
        raise ValueError("correlation_threshold must be in (0, 1]")
    n_columns = len(feature_names)
    columns = [[row[j] for row in feature_rows] for j in range(n_columns)]
    kept: list[int] = []
    for j in range(n_columns):
        redundant = any(
            abs(_pearson(columns[j], columns[k])) > threshold for k in kept
        )
        if not redundant:
            kept.append(j)
    return kept
