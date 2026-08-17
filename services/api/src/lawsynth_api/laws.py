"""Plain-language reading of a stored world's evolution laws.

The domain persists a world declaratively -- one arithmetic expression per state
(``dX/dt = ...``).  This module turns each expression into a ranked, readable
law without evaluating it: native world output is valid arithmetic, so the
standard-library :mod:`ast` parses it safely.  The algorithm is ported from the
Python SDK's ``Study.explain`` law reader so the HTTP ``explain``/``report``
surface agrees with the SDK and CLI, and it depends only on the standard library
(no native runtime, no dataset).
"""

from __future__ import annotations

import ast
from typing import Mapping, Sequence


def _format_coeff(value: float) -> str:
    return f"{value:.4g}"


def _format_feature(factors: Sequence[str]) -> str:
    if not factors:
        return "1"
    counts: dict[str, int] = {}
    for name in factors:
        counts[name] = counts.get(name, 0) + 1
    parts = []
    for name in sorted(counts):
        power = counts[name]
        parts.append(name if power == 1 else f"{name}^{power}")
    return "·".join(parts)


def _product(node: ast.AST, coeff: list[float], factors: list[str]) -> None:
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Mult):
        _product(node.left, coeff, factors)
        _product(node.right, coeff, factors)
    elif isinstance(node, ast.Constant) and isinstance(node.value, (int, float)):
        coeff[0] *= float(node.value)
    elif isinstance(node, ast.Name):
        factors.append(node.id)
    elif isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
        coeff[0] *= -1.0
        _product(node.operand, coeff, factors)
    elif (
        isinstance(node, ast.BinOp)
        and isinstance(node.op, ast.Pow)
        and isinstance(node.left, ast.Name)
        and isinstance(node.right, ast.Constant)
    ):
        factors.extend([node.left.id] * int(node.right.value))
    elif isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div) and isinstance(node.right, ast.Constant):
        _product(node.left, coeff, factors)
        coeff[0] /= float(node.right.value)
    else:  # unexpected node shape -> surface as an opaque factor, never evaluate
        factors.append(ast.dump(node))


def _extract_terms(expression: str) -> tuple[tuple[float, tuple[str, ...]], ...]:
    """Flatten an arithmetic expression into additive (coefficient, factors) terms."""

    tree = ast.parse(expression, mode="eval").body
    terms: list[tuple[float, tuple[str, ...]]] = []

    def walk(node: ast.AST, sign: float) -> None:
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
            walk(node.left, sign)
            walk(node.right, sign)
        elif isinstance(node, ast.BinOp) and isinstance(node.op, ast.Sub):
            walk(node.left, sign)
            walk(node.right, -sign)
        elif isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.UAdd):
            walk(node.operand, sign)
        elif isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
            walk(node.operand, -sign)
        else:
            coeff = [sign]
            factors: list[str] = []
            _product(node, coeff, factors)
            terms.append((coeff[0], tuple(factors)))

    walk(tree, 1.0)
    terms.sort(key=lambda item: -abs(item[0]))
    return tuple(terms)


def read_law(target: str, expression: str) -> dict[str, object]:
    """Return a structured, plain-language reading of one ``dX/dt`` law."""

    try:
        raw = _extract_terms(expression)
    except (SyntaxError, ValueError):
        # Non-arithmetic or unparseable expression: report it verbatim rather
        # than guess a structure that is not there.
        return {
            "target": target,
            "expression": expression,
            "readable": f"d{target}/dt = {expression}",
            "terms": [],
            "dominant_term": None,
        }
    terms = [(coeff, _format_feature(factors)) for coeff, factors in raw]
    pieces: list[str] = []
    for index, (coeff, feature) in enumerate(terms):
        magnitude = _format_coeff(abs(coeff))
        sign = "-" if coeff < 0 else "+"
        chunk = magnitude if feature == "1" else f"{magnitude}·{feature}"
        if index == 0:
            pieces.append(f"-{chunk}" if coeff < 0 else chunk)
        else:
            pieces.append(f"{sign} {chunk}")
    rhs = " ".join(pieces) if pieces else "0"
    return {
        "target": target,
        "expression": expression,
        "readable": f"d{target}/dt = {rhs}",
        "terms": [{"coefficient": coeff, "feature": feature} for coeff, feature in terms],
        "dominant_term": terms[0][1] if terms else None,
    }


def read_laws(equations: Mapping[str, str]) -> list[dict[str, object]]:
    """Read every ``target -> expression`` law, ordered by target name."""

    return [read_law(target, equations[target]) for target in sorted(equations)]


def dependencies(laws: Sequence[Mapping[str, object]], states: Sequence[str]) -> dict[str, list[str]]:
    """Map each law's target to the state variables its features reference."""

    state_set = set(states)
    graph: dict[str, list[str]] = {}
    for law in laws:
        used: set[str] = set()
        for term in law.get("terms", []):  # type: ignore[union-attr]
            feature = term.get("feature", "") if isinstance(term, Mapping) else ""
            tokens = feature.replace("·", " ").replace("^2", "").replace("^3", "").split()
            used.update(token for token in tokens if token in state_set)
        graph[str(law["target"])] = sorted(used)
    return graph


def total_terms(laws: Sequence[Mapping[str, object]]) -> int:
    """Total number of additive terms across all laws (a complexity proxy)."""

    return sum(len(law.get("terms", [])) for law in laws)  # type: ignore[arg-type]
