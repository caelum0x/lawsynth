"""Segment-label recovery scoring for recorded regime assignments."""
from collections.abc import Iterable
from .errors import SchemaError

def accuracy(expected: Iterable[str], recovered: Iterable[str]) -> float:
    truth, found = list(expected), list(recovered)
    if not truth or len(truth) != len(found): raise SchemaError("regime sequences must have equal nonzero length")
    return sum(a == b for a, b in zip(truth, found, strict=True)) / len(truth)
