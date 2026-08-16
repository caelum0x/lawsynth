"""Exact normalized equation recovery scoring."""
import re
from .metrics import precision_recall_f1

def normalize(expression: str) -> str:
    """Normalize whitespace and commutative additive terms without symbolic claims."""
    compact = re.sub(r"\s+", "", expression)
    return "+".join(sorted(compact.split("+")))

def score(expected: str, recovered: str) -> dict[str, float]:
    exact = float(normalize(expected) == normalize(recovered))
    precision, recall, f1 = precision_recall_f1(normalize(expected).split("+"), normalize(recovered).split("+"))
    return {"exact": exact, "term_precision": precision, "term_recall": recall, "term_f1": f1}
