"""Error-rate calculations for failed recorded benchmark runs."""
from collections.abc import Iterable
from .errors import SchemaError

def error_rate(outcomes: Iterable[bool]) -> float:
    values = list(outcomes)
    if not values: raise SchemaError("at least one outcome is required")
    return sum(not value for value in values) / len(values)

def classify(messages: Iterable[str]) -> dict[str, int]:
    result: dict[str, int] = {}
    for message in messages:
        key = message.split(":", 1)[0].strip() or "unknown"
        result[key] = result.get(key, 0) + 1
    return dict(sorted(result.items()))
