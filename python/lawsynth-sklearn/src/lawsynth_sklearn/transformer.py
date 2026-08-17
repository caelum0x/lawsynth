"""LawSynthTransformer — feature-library engineering as an sklearn transformer.

Emits the engineered candidate-library columns LawSynth discovery searches over
(a polynomial library up to ``degree``, optional bias, optional ``sin``/``cos``
augmentation). Optionally applies a deterministic, gplearn-style correlation
prune to return a least-correlated subset for downstream sklearn models.
"""

from __future__ import annotations

from typing import Any

from ._compat import BaseEstimator, HAS_SKLEARN, Tags, TransformerMixin, check_is_fitted
from ._data import as_output, default_feature_names, to_float_rows
from ._features import build_feature_library, correlation_prune


class LawSynthTransformer(TransformerMixin, BaseEstimator):
    """Expand raw features into LawSynth's polynomial/trig candidate library.

    Parameters
    ----------
    degree
        Maximum total polynomial degree.
    include_bias
        Emit a constant ``1`` column.
    include_trigonometric
        Append ``sin``/``cos`` of each raw feature.
    prune_correlation
        When ``True``, drop columns whose absolute Pearson correlation with an
        already-kept column exceeds ``correlation_threshold`` (deterministic,
        left-to-right greedy selection).
    correlation_threshold
        The pruning cutoff in ``(0, 1]``.

    Fitted attributes: ``n_features_in_``, ``feature_names_in_``,
    ``feature_names_out_``, ``kept_indices_``.
    """

    def __init__(
        self,
        *,
        degree: int = 2,
        include_bias: bool = False,
        include_trigonometric: bool = False,
        prune_correlation: bool = False,
        correlation_threshold: float = 0.999,
    ) -> None:
        self.degree = degree
        self.include_bias = include_bias
        self.include_trigonometric = include_trigonometric
        self.prune_correlation = prune_correlation
        self.correlation_threshold = correlation_threshold

    def __sklearn_tags__(self) -> Any:
        if HAS_SKLEARN and hasattr(super(), "__sklearn_tags__"):  # pragma: no cover
            tags = super().__sklearn_tags__()
            tags.non_deterministic = False
            tags.requires_fit = True
            return tags
        return Tags(estimator_type="transformer", requires_fit=True, non_deterministic=False)

    def fit(self, X: Any, y: Any = None) -> "LawSynthTransformer":
        """Compute the feature names and (optionally) the pruned column subset."""
        rows, given_names = to_float_rows(X)
        names = default_feature_names(len(rows[0]), given_names)
        feature_rows, feature_names = build_feature_library(
            rows,
            names,
            degree=self.degree,
            include_bias=self.include_bias,
            include_trigonometric=self.include_trigonometric,
        )
        if self.prune_correlation:
            kept = correlation_prune(feature_rows, feature_names, self.correlation_threshold)
        else:
            kept = list(range(len(feature_names)))

        self.n_features_in_ = len(names)
        self.feature_names_in_ = tuple(names)
        self.kept_indices_ = tuple(kept)
        self.feature_names_out_ = tuple(feature_names[i] for i in kept)
        return self

    def transform(self, X: Any) -> Any:
        """Return the engineered (and optionally pruned) feature matrix."""
        check_is_fitted(self)
        rows, _ = to_float_rows(X)
        if len(rows[0]) != self.n_features_in_:
            raise ValueError(
                f"X has {len(rows[0])} features; this transformer was fit on {self.n_features_in_}"
            )
        feature_rows, _ = build_feature_library(
            rows,
            list(self.feature_names_in_),
            degree=self.degree,
            include_bias=self.include_bias,
            include_trigonometric=self.include_trigonometric,
        )
        selected = [[row[i] for i in self.kept_indices_] for row in feature_rows]
        return as_output(selected)

    def get_feature_names_out(self, input_features: Any = None) -> Any:
        """Names of the emitted columns (sklearn ``get_feature_names_out``)."""
        check_is_fitted(self)
        names = list(self.feature_names_out_)
        if HAS_SKLEARN:  # pragma: no cover - depends on environment
            import numpy as np

            return np.asarray(names, dtype=object)
        return names
