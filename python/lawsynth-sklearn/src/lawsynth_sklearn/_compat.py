"""scikit-learn compatibility layer with graceful degradation.

The estimators in this package are *real* scikit-learn estimators when sklearn
is installed: they inherit :class:`~sklearn.base.BaseEstimator` and the relevant
mixins, so they drop into ``Pipeline``/``GridSearchCV``/``clone`` unchanged.

When sklearn is **not** installed the same public contract is provided by
standalone fall-backs defined here — a ``BaseEstimator`` clone that implements
``get_params``/``set_params`` by introspecting ``__init__`` exactly the way
sklearn does, a ``NotFittedError``, and ``check_is_fitted``. This keeps the
package importable and usable in a pure-standard-library, offline environment.
"""

from __future__ import annotations

import inspect
from dataclasses import dataclass, field
from typing import Any

try:  # pragma: no cover - exercised by whichever environment runs the suite
    from sklearn.base import BaseEstimator as _SkBaseEstimator
    from sklearn.base import RegressorMixin as _SkRegressorMixin
    from sklearn.base import TransformerMixin as _SkTransformerMixin
    from sklearn.exceptions import NotFittedError as _SkNotFittedError

    HAS_SKLEARN = True
except Exception:  # pragma: no cover - environment without sklearn
    HAS_SKLEARN = False


__all__ = [
    "HAS_SKLEARN",
    "BaseEstimator",
    "RegressorMixin",
    "TransformerMixin",
    "NotFittedError",
    "check_is_fitted",
    "Tags",
]


if HAS_SKLEARN:
    BaseEstimator = _SkBaseEstimator
    RegressorMixin = _SkRegressorMixin
    TransformerMixin = _SkTransformerMixin
    NotFittedError = _SkNotFittedError

else:

    class NotFittedError(ValueError, AttributeError):  # type: ignore[no-redef]
        """Raised when an estimator method is used before ``fit``.

        Mirrors :class:`sklearn.exceptions.NotFittedError` (same MRO of
        ``ValueError`` + ``AttributeError``) so downstream ``except`` clauses
        behave identically whether or not sklearn is installed.
        """

    class BaseEstimator:  # type: ignore[no-redef]
        """Standalone stand-in for :class:`sklearn.base.BaseEstimator`.

        Implements the ``get_params`` / ``set_params`` contract by inspecting
        the estimator's ``__init__`` signature, identical in behaviour to
        sklearn so ``clone``-style round-trips reproduce the estimator exactly.
        """

        @classmethod
        def _get_param_names(cls) -> list[str]:
            init = cls.__init__
            if init is object.__init__:
                return []
            signature = inspect.signature(init)
            params = []
            for name, parameter in signature.parameters.items():
                if name == "self":
                    continue
                if parameter.kind == parameter.VAR_POSITIONAL:
                    raise RuntimeError(
                        f"{cls.__name__} must not use *args in __init__ to satisfy "
                        "the scikit-learn estimator contract"
                    )
                if parameter.kind == parameter.VAR_KEYWORD:
                    continue
                params.append(name)
            return sorted(params)

        def get_params(self, deep: bool = True) -> dict[str, Any]:
            out: dict[str, Any] = {}
            for key in self._get_param_names():
                value = getattr(self, key)
                if deep and hasattr(value, "get_params") and not isinstance(value, type):
                    for sub_key, sub_value in value.get_params().items():
                        out[f"{key}__{sub_key}"] = sub_value
                out[key] = value
            return out

        def set_params(self, **params: Any) -> "BaseEstimator":
            if not params:
                return self
            valid = self.get_params(deep=True)
            nested: dict[str, dict[str, Any]] = {}
            for key, value in params.items():
                head, _, tail = key.partition("__")
                if head not in self._get_param_names():
                    raise ValueError(
                        f"Invalid parameter {key!r} for estimator "
                        f"{type(self).__name__}. Valid parameters are: "
                        f"{self._get_param_names()!r}."
                    )
                if tail:
                    nested.setdefault(head, {})[tail] = value
                else:
                    setattr(self, head, value)
            for head, sub_params in nested.items():
                getattr(self, head).set_params(**sub_params)
            return self

        def __repr__(self) -> str:
            params = self.get_params(deep=False)
            body = ", ".join(f"{k}={v!r}" for k, v in sorted(params.items()))
            return f"{type(self).__name__}({body})"

    class RegressorMixin:  # type: ignore[no-redef]
        """Standalone stand-in for :class:`sklearn.base.RegressorMixin`."""

        _estimator_type = "regressor"

        def score(self, X: Any, y: Any, sample_weight: Any = None) -> float:
            """Coefficient of determination R² of ``predict(X)`` against ``y``."""
            from ._data import to_float_rows

            prediction = list(self.predict(X))  # type: ignore[attr-defined]
            truth_rows, _ = to_float_rows(y, allow_1d=True)
            truth = [row[0] for row in truth_rows]
            return _r2_score(truth, [float(value) for value in prediction])

    class TransformerMixin:  # type: ignore[no-redef]
        """Standalone stand-in for :class:`sklearn.base.TransformerMixin`."""

        def fit_transform(self, X: Any, y: Any = None, **fit_params: Any) -> Any:
            return self.fit(X, y, **fit_params).transform(X)  # type: ignore[attr-defined]


@dataclass
class Tags:
    """A minimal, sklearn-shaped tag bundle used by the standalone path.

    When sklearn is installed the estimators delegate ``__sklearn_tags__`` to
    the framework; when it is absent they return one of these so callers can
    still introspect ``estimator_type`` / ``requires_fit`` / ``non_deterministic``.
    """

    estimator_type: str | None = None
    requires_fit: bool = True
    non_deterministic: bool = False
    target_tags: dict[str, Any] = field(default_factory=dict)
    transformer_tags: dict[str, Any] | None = None
    regressor_tags: dict[str, Any] | None = None


def _r2_score(truth: list[float], prediction: list[float]) -> float:
    """Plain-stdlib R² (matches sklearn's ``r2_score`` default reduction)."""
    count = min(len(truth), len(prediction))
    if count == 0:
        return 0.0
    truth = truth[:count]
    prediction = prediction[:count]
    mean = sum(truth) / count
    ss_tot = sum((value - mean) ** 2 for value in truth)
    ss_res = sum((t - p) ** 2 for t, p in zip(truth, prediction))
    if ss_tot == 0.0:
        # Constant target: perfect only if residual is zero, else undefined→0.
        return 1.0 if ss_res == 0.0 else 0.0
    return 1.0 - ss_res / ss_tot


def check_is_fitted(estimator: Any, attributes: Any = None) -> None:
    """Raise :class:`NotFittedError` unless the estimator has been fitted.

    Prefers sklearn's own implementation when available (so behaviour matches
    the framework exactly); otherwise applies the same rule — an estimator is
    fitted iff it carries at least one public attribute ending in ``_``.
    """
    if HAS_SKLEARN:  # pragma: no cover - depends on environment
        from sklearn.utils.validation import check_is_fitted as _sk_check

        _sk_check(estimator, attributes=attributes)
        return
    if attributes is not None:
        names = [attributes] if isinstance(attributes, str) else list(attributes)
        fitted = all(hasattr(estimator, name) for name in names)
    else:
        fitted = any(
            name.endswith("_") and not name.startswith("__")
            for name in vars(estimator)
        )
    if not fitted:
        raise NotFittedError(
            f"This {type(estimator).__name__} instance is not fitted yet. Call "
            "'fit' with appropriate arguments before using this estimator."
        )
