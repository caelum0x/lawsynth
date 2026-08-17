"""Input/output coercion — accept lists, numpy arrays, or pandas frames.

The estimators avoid a hard numpy dependency so they stay importable and usable
in a pure-standard-library environment, yet interoperate cleanly with numpy /
pandas when those are present (as they always are alongside scikit-learn).
"""

from __future__ import annotations

from typing import Any


def _has_numpy() -> bool:
    try:  # pragma: no cover - trivial
        import numpy  # noqa: F401

        return True
    except Exception:  # pragma: no cover
        return False


def _is_dataframe(X: Any) -> bool:
    return (
        hasattr(X, "columns")
        and hasattr(X, "to_numpy")
        and type(X).__name__ == "DataFrame"
    )


def to_float_rows(X: Any, *, allow_1d: bool = False) -> tuple[list[list[float]], list[str] | None]:
    """Coerce array-like ``X`` into ``(rows, column_names_or_None)``.

    ``rows`` is a list of equal-length lists of floats (n_samples × n_features).
    Column names are recovered from a pandas ``DataFrame`` when possible, else
    ``None``. A 1-D input is treated as a single column when ``allow_1d`` is set.
    """
    if X is None:
        raise ValueError("input array is required (got None)")

    names: list[str] | None = None

    if _is_dataframe(X):
        names = [str(column) for column in X.columns]
        X = X.to_numpy()

    # numpy array → nested python lists
    if hasattr(X, "tolist") and not isinstance(X, (list, tuple)):
        X = X.tolist()

    if not isinstance(X, (list, tuple)):
        raise ValueError(f"cannot interpret input of type {type(X).__name__!r} as an array")

    sequence = list(X)
    if not sequence:
        raise ValueError("input array is empty")

    first = sequence[0]
    is_2d = isinstance(first, (list, tuple)) or (
        hasattr(first, "__len__") and not isinstance(first, (str, bytes))
    )

    if not is_2d:
        if not allow_1d:
            raise ValueError(
                "expected a 2-D array (n_samples, n_features); got a 1-D sequence"
            )
        rows = [[float(value)] for value in sequence]
        return rows, names

    rows = [[float(value) for value in row] for row in sequence]
    width = len(rows[0])
    if any(len(row) != width for row in rows):
        raise ValueError("all rows must have the same number of features")
    return rows, names


def rows_to_columns(rows: list[list[float]], names: list[str]) -> dict[str, list[float]]:
    """Transpose row-major samples into ``{name: column}`` for the SDK Dataset."""
    columns: dict[str, list[float]] = {name: [] for name in names}
    for row in rows:
        for name, value in zip(names, row):
            columns[name].append(value)
    return columns


def default_feature_names(n_features: int, given: list[str] | None) -> list[str]:
    """Return valid identifier column names (given names, or ``x0..xk``)."""
    if given is not None:
        names = [str(name) for name in given]
        if len(names) != n_features:
            raise ValueError(
                f"got {len(names)} feature names for {n_features} columns"
            )
        for name in names:
            if not name.isidentifier():
                raise ValueError(
                    f"feature name {name!r} is not a valid identifier; the LawSynth "
                    "engine requires identifier-safe state names"
                )
        return names
    return [f"x{i}" for i in range(n_features)]


def as_output(rows: list[list[float]]) -> Any:
    """Return a numpy 2-D array when numpy is available, else a list of lists."""
    if _has_numpy():  # pragma: no cover - depends on environment
        import numpy as np

        return np.asarray(rows, dtype=float)
    return rows


def as_output_1d(values: list[float]) -> Any:
    """Return a numpy 1-D array when numpy is available, else a list."""
    if _has_numpy():  # pragma: no cover - depends on environment
        import numpy as np

        return np.asarray(values, dtype=float)
    return values


def time_vector(t: Any, n_samples: int) -> list[float]:
    """Resolve a strictly-increasing time vector of length ``n_samples``.

    ``t`` may be ``None`` (defaults to ``0, 1, ... n-1``), a scalar sampling
    interval, or an explicit array-like of timestamps.
    """
    if t is None:
        return [float(i) for i in range(n_samples)]
    if isinstance(t, (int, float)):
        dt = float(t)
        return [i * dt for i in range(n_samples)]
    if hasattr(t, "tolist"):
        t = t.tolist()
    values = [float(value) for value in t]
    if len(values) != n_samples:
        raise ValueError(
            f"time vector has length {len(values)} but X has {n_samples} samples"
        )
    return values
