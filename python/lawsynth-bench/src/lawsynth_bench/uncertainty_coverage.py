"""Empirical interval coverage of recorded predictions."""
from collections.abc import Iterable
from .errors import SchemaError

def coverage(observed: Iterable[float], lower: Iterable[float], upper: Iterable[float]) -> float:
    triples = list(zip(observed, lower, upper, strict=True))
    if not triples or any(low > high for _, low, high in triples): raise SchemaError("invalid coverage intervals")
    return sum(low <= value <= high for value, low, high in triples) / len(triples)
