"""Bridge the ``lawsynth_connectors`` library into validated LawSynth datasets.

The connectors package deliberately performs *no* numeric coercion: an external
source that cannot represent a number faithfully is surfaced verbatim (a CSV
cell arrives as ``"1.5"``, not ``1.5``).  Coercing raw records into the finite
floats the discovery core requires is a product concern, so it happens *here*,
at the SDK boundary, exactly once.

Usage::

    ds = lawsynth.load_source(
        "filesystem", "obs.csv",
        time="t", state=["x", "y"], options={"root": "."},
    )
    study = lawsynth.Study.from_dataset(ds, state=["x", "y"])

The heavy connectors dependency is imported lazily so the core SDK keeps
importing even when ``lawsynth-connectors`` (or an optional driver) is absent;
a missing driver degrades to a single, clear :class:`SourceError`.
"""

from __future__ import annotations

import math
from collections.abc import Mapping, Sequence
from decimal import Decimal
from pathlib import PurePath
from typing import Any

from .dataset import Dataset
from .errors import LawSynthError, ValidationError

__all__ = ["SourceError", "load_source", "read_source_records"]


class SourceError(LawSynthError):
    """Raised when a data source cannot be imported into a LawSynth dataset."""


# --------------------------------------------------------------------------- #
# Numeric coercion — the boundary where typed/string records become floats     #
# --------------------------------------------------------------------------- #


def _coerce_float(value: Any, column: str, row_index: int) -> float:
    """Faithfully coerce one connector cell into a finite float.

    Connectors hand back whatever the source encoded — strings from CSV/HTTP,
    native ints/floats from typed backends.  This is the single place that
    turns those into the finite floats discovery requires, rejecting anything
    that cannot be represented as a real observation.
    """
    if isinstance(value, bool):
        raise SourceError(
            f"column {column!r} row {row_index}: boolean is not a numeric observation"
        )
    if isinstance(value, (int, float, Decimal)):
        result = float(value)
    elif isinstance(value, str):
        text = value.strip()
        if not text:
            raise SourceError(
                f"column {column!r} row {row_index}: empty cell cannot be coerced to a number"
            )
        try:
            result = float(text)
        except ValueError:
            raise SourceError(
                f"column {column!r} row {row_index}: {value!r} is not numeric"
            ) from None
    elif value is None:
        raise SourceError(f"column {column!r} row {row_index}: missing value")
    else:
        raise SourceError(
            f"column {column!r} row {row_index}: unsupported value type "
            f"{type(value).__name__!r}"
        )
    if not math.isfinite(result):
        raise SourceError(
            f"column {column!r} row {row_index}: non-finite value {value!r}"
        )
    return result


# --------------------------------------------------------------------------- #
# Input validation                                                             #
# --------------------------------------------------------------------------- #


def _validate_selection(time: str, state: Sequence[str]) -> tuple[str, tuple[str, ...]]:
    if not isinstance(time, str) or not time.strip():
        raise ValidationError("time must be a non-empty column name")
    states = tuple(state)
    if not states:
        raise ValidationError("at least one state column is required")
    for name in states:
        if not isinstance(name, str) or not name.strip():
            raise ValidationError("state column names must be non-empty strings")
    if len(set(states)) != len(states):
        raise ValidationError(f"duplicate state columns: {list(states)}")
    if time in states:
        raise ValidationError(f"time column {time!r} cannot also be a state column")
    return time, states


def _default_name(kind: str, resource: str) -> str:
    stem = PurePath(resource).stem or resource
    return f"{kind}:{stem}"


# --------------------------------------------------------------------------- #
# Connector orchestration                                                      #
# --------------------------------------------------------------------------- #


def read_source_records(
    kind: str,
    resource: str,
    *,
    columns: Sequence[str],
    options: Mapping[str, Any] | None = None,
    credentials: Any = None,
    batch_size: int | None = None,
    max_rows: int | None = None,
) -> tuple[Mapping[str, Any], ...]:
    """Create a connector via the registry and read a projection in batches.

    Returns the raw (uncoerced) records exactly as the connector produced them.
    Optional-dependency connectors that cannot import their driver, and unknown
    connector kinds, degrade to a clear :class:`SourceError`.
    """
    try:
        from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
        from lawsynth_connectors.errors import (
            ConnectorError,
            DependencyUnavailableError,
        )
    except ImportError as error:  # pragma: no cover - packaging guard
        raise SourceError(
            "the lawsynth-connectors package is not importable; install it to load "
            "data from external sources"
        ) from error

    config_kwargs: dict[str, Any] = {"name": kind, "options": dict(options or {})}
    if batch_size is not None:
        config_kwargs["batch_size"] = batch_size
    if max_rows is not None:
        config_kwargs["max_rows"] = max_rows

    try:
        config = ConnectorConfig(**config_kwargs)
    except ConnectorError as error:
        raise SourceError(f"invalid source configuration: {error}") from error

    try:
        connector = (
            registry.create(config, credentials)
            if credentials is not None
            else registry.create(config)
        )
    except DependencyUnavailableError as error:
        raise SourceError(
            f"connector {kind!r} needs an optional driver that is not installed: {error}"
        ) from error
    except ConnectorError as error:
        raise SourceError(f"could not create connector {kind!r}: {error}") from error

    records: list[Mapping[str, Any]] = []
    try:
        connector.connect()
        request = ReadRequest(resource=resource, columns=tuple(columns))
        for batch in connector.read(request):
            records.extend(batch.records)
    except DependencyUnavailableError as error:
        raise SourceError(
            f"connector {kind!r} needs an optional driver that is not installed: {error}"
        ) from error
    except ConnectorError as error:
        raise SourceError(f"failed to read from {kind!r} source: {error}") from error
    finally:
        try:
            connector.close()
        except Exception:  # pragma: no cover - close is best-effort
            pass
    return tuple(records)


def load_source(
    kind: str,
    resource: str,
    *,
    time: str,
    state: Sequence[str],
    options: Mapping[str, Any] | None = None,
    credentials: Any = None,
    batch_size: int | None = None,
    max_rows: int | None = None,
) -> Dataset:
    """Load observations from any connector into a validated numeric ``Dataset``.

    ``kind`` names a registered connector ("filesystem", "http", "s3", ...),
    ``resource`` is the connector-relative locator, ``time`` names the time
    column, and ``state`` names the numeric state columns to model.  Connector
    behaviour is tuned through ``options`` (for example ``{"root": "."}`` for
    the filesystem connector or ``{"allow_private_network": True}`` for HTTP).

    Records are read in bounded batches, coerced to finite floats at this
    boundary, and assembled into a :class:`~lawsynth.dataset.Dataset` whose
    invariants (strictly increasing time, aligned finite columns) are enforced
    on construction.  The returned dataset is accepted directly by
    :meth:`lawsynth.Study.from_dataset`.
    """
    if not isinstance(kind, str) or not kind.strip():
        raise ValidationError("source kind must be a non-empty string")
    if not isinstance(resource, str) or not resource.strip():
        raise ValidationError("source resource must be a non-empty string")
    time_column, state_columns = _validate_selection(time, state)

    records = read_source_records(
        kind,
        resource,
        columns=(time_column, *state_columns),
        options=options,
        credentials=credentials,
        batch_size=batch_size,
        max_rows=max_rows,
    )
    if not records:
        raise SourceError(f"source {kind!r} resource {resource!r} returned no records")

    required = (time_column, *state_columns)
    times: list[float] = []
    columns: dict[str, list[float]] = {name: [] for name in state_columns}
    for row_index, record in enumerate(records):
        missing = [name for name in required if name not in record]
        if missing:
            raise SourceError(
                f"record {row_index} is missing required columns {missing}; "
                f"found {sorted(record)}"
            )
        times.append(_coerce_float(record[time_column], time_column, row_index))
        for name in state_columns:
            columns[name].append(_coerce_float(record[name], name, row_index))

    try:
        return Dataset.from_columns(times, columns)
    except ValidationError as error:
        raise SourceError(
            f"source {kind!r} resource {resource!r} is not a valid time series: {error}"
        ) from error
