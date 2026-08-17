"""Serialize a discovered world into the self-contained explorer JS payload.

A native/SDK world exposes its laws as arithmetic strings with the discovered
coefficients baked in (e.g. ``((1.04*x)+(-0.39*(x*y)))``). To make the world
*interactive*, this module factors those coefficients out into named
parameters: each additive term ``coeff * feature`` becomes ``p * feature`` with
``p`` exposed as a draggable slider. Dragging a slider therefore re-weights a
term and the browser re-integrates the system live.

The produced payload is plain JSON-serialisable data — states, parameters
(id/value/label/bounds), parameterised laws, initial conditions and time
bounds — everything the embedded integrator needs, with no kernel round-trip.
"""

from __future__ import annotations

import ast
from collections.abc import Mapping, Sequence
from typing import Any

from .errors import ArtifactValidationError
from .explorer_math import INTEGRATION_METHODS, parse_expression

__all__ = ["build_payload", "flatten_terms"]

_MIDDLE_DOT = "·"  # "·" used for readable feature products


def flatten_terms(expression: str) -> list[tuple[float, tuple[str, ...]]]:
    """Flatten an arithmetic expression into additive ``(coeff, factors)`` terms.

    ``factors`` is the sorted tuple of variable names multiplied together (a
    squared variable appears twice). Parsing uses :mod:`ast` without evaluation.
    """
    tree = parse_expression(expression)
    terms: list[tuple[float, tuple[str, ...]]] = []

    def walk(node: ast.expr, sign: float) -> None:
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
            product(node, coeff, factors)
            terms.append((coeff[0], tuple(sorted(factors))))

    def product(node: ast.expr, coeff: list[float], factors: list[str]) -> None:
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Mult):
            product(node.left, coeff, factors)
            product(node.right, coeff, factors)
        elif isinstance(node, ast.Constant) and isinstance(node.value, (int, float)) and not isinstance(node.value, bool):
            coeff[0] *= float(node.value)
        elif isinstance(node, ast.Name):
            factors.append(node.id)
        elif isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
            coeff[0] *= -1.0
            product(node.operand, coeff, factors)
        elif isinstance(node, ast.BinOp) and isinstance(node.op, ast.Pow) and isinstance(node.left, ast.Name) and isinstance(node.right, ast.Constant):
            factors.extend([node.left.id] * int(node.right.value))
        elif isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div) and isinstance(node.right, ast.Constant):
            product(node.left, coeff, factors)
            coeff[0] /= float(node.right.value)
        else:
            raise ArtifactValidationError(f"unsupported equation structure in {ast.dump(node)}")

    walk(tree, 1.0)
    # Deterministic order: shorter features first, then lexicographic.
    terms.sort(key=lambda item: (len(item[1]), item[1]))
    return terms


def _feature_expr(factors: Sequence[str]) -> str:
    return "*".join(factors) if factors else ""


def _feature_label(factors: Sequence[str]) -> str:
    if not factors:
        return "1"
    counts: dict[str, int] = {}
    for name in factors:
        counts[name] = counts.get(name, 0) + 1
    parts = [name if power == 1 else f"{name}^{power}" for name, power in sorted(counts.items())]
    return _MIDDLE_DOT.join(parts)


def _bounds(value: float) -> tuple[float, float, float]:
    """A symmetric slider range around a coefficient that permits a sign flip."""
    if value == 0.0:
        return -1.0, 1.0, 0.01
    magnitude = abs(value)
    low = value - 2.0 * magnitude
    high = value + 2.0 * magnitude
    return low, high, (high - low) / 200.0


def _parameterise(states: Sequence[str], equations: Mapping[str, str]) -> tuple[list[dict[str, Any]], dict[str, str]]:
    parameters: list[dict[str, Any]] = []
    laws: dict[str, str] = {}
    for target in states:
        expression = equations.get(target)
        if not isinstance(expression, str) or not expression.strip():
            raise ArtifactValidationError(f"missing equation for state {target!r}")
        terms = flatten_terms(expression)
        pieces: list[str] = []
        for index, (coeff, factors) in enumerate(terms):
            param_id = f"k_{target}_{index}"
            low, high, step = _bounds(coeff)
            feature = _feature_expr(factors)
            parameters.append({
                "id": param_id,
                "value": coeff,
                "label": _feature_label(factors),
                "target": target,
                "min": low,
                "max": high,
                "step": step,
            })
            pieces.append(f"({param_id}*({feature}))" if feature else f"({param_id})")
        laws[target] = "+".join(pieces) if pieces else "(0)"
    return parameters, laws


def build_payload(
    *,
    name: str,
    states: Sequence[str],
    equations: Mapping[str, str],
    initial: Mapping[str, float],
    start: float,
    end: float,
    step: float,
    method: str = "rk4",
    time_symbol: str = "t",
) -> dict[str, Any]:
    """Build the JSON-serialisable payload driving the embedded integrator."""
    state_list = list(states)
    if not state_list:
        raise ArtifactValidationError("a world needs at least one state variable")
    if any(not isinstance(s, str) or not s.isidentifier() for s in state_list):
        raise ArtifactValidationError("state names must be identifiers")
    if method not in INTEGRATION_METHODS:
        raise ArtifactValidationError(f"method must be one of {INTEGRATION_METHODS}")
    for label, value in (("start", start), ("end", end), ("step", step)):
        if not isinstance(value, (int, float)) or isinstance(value, bool):
            raise ArtifactValidationError(f"{label} must be a number")
    if end <= start or step <= 0:
        raise ArtifactValidationError("time range must be increasing with a positive step")

    parameters, laws = _parameterise(state_list, equations)
    initial_state = {state: float(initial.get(state, 0.0)) for state in state_list}
    return {
        "name": str(name),
        "states": state_list,
        "parameters": parameters,
        "laws": laws,
        "initial": initial_state,
        "time": {"start": float(start), "end": float(end), "step": float(step)},
        "method": method,
        "timeSymbol": time_symbol,
    }
