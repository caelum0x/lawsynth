"""Directed-edge recovery scoring."""
from collections.abc import Iterable
from .metrics import precision_recall_f1

def score(expected: Iterable[tuple[str, str]], recovered: Iterable[tuple[str, str]]) -> dict[str, float]:
    encode = lambda edges: [f"{source}->{target}" for source, target in edges]
    precision, recall, f1 = precision_recall_f1(encode(expected), encode(recovered))
    return {"precision": precision, "recall": recall, "f1": f1}
