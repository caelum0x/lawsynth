"""LawSynthTransformer: feature-library engineering and correlation prune."""

from __future__ import annotations

import math

import pytest

from lawsynth_sklearn import LawSynthTransformer


@pytest.fixture
def simple():
    return [[1.0, 2.0], [2.0, 1.0], [3.0, 0.5], [0.5, 3.0], [4.0, 2.0]]


def _rows(Z):
    return Z.tolist() if hasattr(Z, "tolist") else Z


def test_polynomial_names_and_count(simple):
    tr = LawSynthTransformer(degree=2, include_bias=True)
    Z = tr.fit_transform(simple)
    names = list(tr.get_feature_names_out())
    # bias + 2 linear + 3 quadratic = 6
    assert names == ["1", "x0", "x1", "x0^2", "x0 x1", "x1^2"]
    assert len(_rows(Z)[0]) == 6


def test_no_bias(simple):
    tr = LawSynthTransformer(degree=2, include_bias=False).fit(simple)
    assert "1" not in tr.feature_names_out_


def test_trigonometric_augmentation(simple):
    tr = LawSynthTransformer(degree=1, include_bias=False, include_trigonometric=True)
    Z = _rows(tr.fit_transform(simple))
    names = list(tr.get_feature_names_out())
    assert "sin(x0)" in names and "cos(x1)" in names
    j = names.index("sin(x0)")
    assert abs(Z[0][j] - math.sin(1.0)) < 1e-12


def test_values_are_correct(simple):
    tr = LawSynthTransformer(degree=2, include_bias=True)
    Z = _rows(tr.fit_transform(simple))
    # row [1,2] -> [1, 1, 2, 1, 2, 4]
    assert Z[0] == [1.0, 1.0, 2.0, 1.0, 2.0, 4.0]


def test_transform_matches_fit_transform(simple):
    tr = LawSynthTransformer(degree=2, include_bias=True)
    a = _rows(tr.fit_transform(simple))
    b = _rows(tr.transform(simple))
    assert a == b


def test_correlation_prune_reduces_columns(simple):
    full = LawSynthTransformer(degree=2, include_bias=True).fit(simple)
    pruned = LawSynthTransformer(
        degree=2, include_bias=True, prune_correlation=True, correlation_threshold=0.9
    ).fit(simple)
    assert len(pruned.feature_names_out_) <= len(full.feature_names_out_)
    # kept indices are a subset of the full library, in ascending order.
    assert list(pruned.kept_indices_) == sorted(pruned.kept_indices_)


def test_prune_is_deterministic(simple):
    a = LawSynthTransformer(degree=2, prune_correlation=True, correlation_threshold=0.95).fit(simple)
    b = LawSynthTransformer(degree=2, prune_correlation=True, correlation_threshold=0.95).fit(simple)
    assert a.kept_indices_ == b.kept_indices_


def test_invalid_correlation_threshold(simple):
    with pytest.raises(ValueError):
        LawSynthTransformer(prune_correlation=True, correlation_threshold=1.5).fit(simple)


def test_transform_feature_count_guard(simple):
    tr = LawSynthTransformer(degree=2).fit(simple)
    with pytest.raises(ValueError):
        tr.transform([[1.0, 2.0, 3.0]])
