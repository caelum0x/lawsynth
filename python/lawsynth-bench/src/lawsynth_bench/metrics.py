"""Small dependency-free statistical and recovery metrics."""
from collections.abc import Iterable
import math
from .errors import SchemaError

def _values(values: Iterable[float]) -> list[float]:
    result = [float(v) for v in values]
    if not result or not all(math.isfinite(v) for v in result):
        raise SchemaError("metrics require at least one finite value")
    return result

def mean(values: Iterable[float]) -> float:
    data = _values(values); return math.fsum(data) / len(data)

def median(values: Iterable[float]) -> float:
    data = sorted(_values(values)); middle = len(data) // 2
    return data[middle] if len(data) % 2 else (data[middle - 1] + data[middle]) / 2

def rmse(expected: Iterable[float], actual: Iterable[float]) -> float:
    pairs = list(zip(expected, actual, strict=True))
    if not pairs: raise SchemaError("rmse requires values")
    return math.sqrt(mean((a - b) ** 2 for a, b in pairs))

def mae(expected: Iterable[float], actual: Iterable[float]) -> float:
    pairs = list(zip(expected, actual, strict=True))
    if not pairs: raise SchemaError("mae requires values")
    return mean(abs(a - b) for a, b in pairs)

def precision_recall_f1(expected: Iterable[str], actual: Iterable[str]) -> tuple[float, float, float]:
    truth, predicted = set(expected), set(actual)
    tp = len(truth & predicted); precision = tp / len(predicted) if predicted else 1.0
    recall = tp / len(truth) if truth else 1.0
    return precision, recall, 2 * precision * recall / (precision + recall) if precision + recall else 0.0
