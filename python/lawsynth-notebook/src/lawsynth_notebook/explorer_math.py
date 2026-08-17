"""A tiny, safe expression evaluator and ODE integrator.

This is the Python twin of the JavaScript integrator shipped inside the
``WorldExplorer`` bundle (see :mod:`lawsynth_notebook.explorer_assets`). Both
evaluate the *same* parameterised law expressions over the *same* RK4/Euler
scheme, so the server-rendered initial trajectory and the browser-side
re-simulation agree. Expressions are parsed with the standard-library
:mod:`ast` in ``eval`` mode and walked node-by-node — nothing is ever executed
with :func:`eval`, so untrusted world strings cannot run arbitrary code.

Supported operations mirror the Rust/TS evaluators: constants, variable
symbols, ``add``/``sub``/``mul``/``div``/``pow``, unary negation, and the unary
functions ``abs``/``exp``/``log``/``sqrt``/``sin``/``cos``/``tan`` plus the
binary ``min``/``max``.
"""

from __future__ import annotations

import ast
import math
from collections.abc import Mapping, Sequence
from typing import Any

from .errors import ArtifactValidationError

__all__ = ["evaluate", "integrate", "parse_expression", "INTEGRATION_METHODS"]

INTEGRATION_METHODS = ("rk4", "euler")

# Bound the sample count so a mistyped step cannot allocate unbounded memory.
_MAX_SAMPLES = 20_000

_UNARY_FUNCS: dict[str, Any] = {
    "abs": abs,
    "exp": math.exp,
    "log": math.log,
    "sqrt": math.sqrt,
    "sin": math.sin,
    "cos": math.cos,
    "tan": math.tan,
    "neg": lambda value: -value,
}
_BINARY_FUNCS: dict[str, Any] = {
    "min": min,
    "max": max,
    "pow": math.pow,
}


def parse_expression(expression: str) -> ast.expr:
    """Parse an arithmetic expression string into a validated AST node."""
    if not isinstance(expression, str) or not expression.strip():
        raise ArtifactValidationError("law expression must be a non-empty string")
    try:
        return ast.parse(expression, mode="eval").body
    except SyntaxError as error:
        raise ArtifactValidationError(f"cannot parse expression {expression!r}: {error}") from error


def evaluate(node: ast.expr, scope: Mapping[str, float]) -> float:
    """Evaluate a parsed expression node against a numeric ``scope``."""
    if isinstance(node, ast.Constant):
        if isinstance(node.value, bool) or not isinstance(node.value, (int, float)):
            raise ArtifactValidationError("only numeric constants are allowed")
        return float(node.value)
    if isinstance(node, ast.Name):
        try:
            return float(scope[node.id])
        except KeyError as error:
            raise ArtifactValidationError(f"unknown symbol {node.id!r}") from error
    if isinstance(node, ast.UnaryOp):
        operand = evaluate(node.operand, scope)
        if isinstance(node.op, ast.USub):
            return -operand
        if isinstance(node.op, ast.UAdd):
            return operand
        raise ArtifactValidationError("unsupported unary operator")
    if isinstance(node, ast.BinOp):
        left = evaluate(node.left, scope)
        right = evaluate(node.right, scope)
        if isinstance(node.op, ast.Add):
            return left + right
        if isinstance(node.op, ast.Sub):
            return left - right
        if isinstance(node.op, ast.Mult):
            return left * right
        if isinstance(node.op, ast.Div):
            return left / right
        if isinstance(node.op, ast.Pow):
            return float(left ** right)
        raise ArtifactValidationError("unsupported binary operator")
    if isinstance(node, ast.Call):
        if node.keywords or not isinstance(node.func, ast.Name):
            raise ArtifactValidationError("only simple function calls are allowed")
        name = node.func.id
        args = [evaluate(arg, scope) for arg in node.args]
        if name in _UNARY_FUNCS and len(args) == 1:
            return float(_UNARY_FUNCS[name](args[0]))
        if name in _BINARY_FUNCS and len(args) == 2:
            return float(_BINARY_FUNCS[name](args[0], args[1]))
        raise ArtifactValidationError(f"unsupported function call {name!r}")
    raise ArtifactValidationError(f"unsupported expression node {type(node).__name__}")


def _derivatives(
    compiled: Mapping[str, ast.expr],
    states: Sequence[str],
    params: Mapping[str, float],
    values: Mapping[str, float],
    time_symbol: str,
    time: float,
) -> dict[str, float]:
    scope: dict[str, float] = {**params, **values, time_symbol: time}
    return {state: evaluate(compiled[state], scope) for state in states}


def integrate(
    states: Sequence[str],
    laws: Mapping[str, str],
    params: Mapping[str, float],
    initial: Mapping[str, float],
    *,
    start: float,
    end: float,
    step: float,
    method: str = "rk4",
    time_symbol: str = "t",
) -> dict[str, Any]:
    """Integrate the law system and return ``{"time": [...], "values": {...}}``.

    A fixed-step RK4 (default) or explicit Euler scheme, matching the embedded
    JavaScript integrator sample-for-sample.
    """
    if method not in INTEGRATION_METHODS:
        raise ArtifactValidationError(f"method must be one of {INTEGRATION_METHODS}")
    if not (math.isfinite(start) and math.isfinite(end) and math.isfinite(step)):
        raise ArtifactValidationError("time bounds must be finite")
    if end <= start or step <= 0:
        raise ArtifactValidationError("time range must be increasing with a positive step")
    sample_count = int((end - start) / step) + 1
    if sample_count > _MAX_SAMPLES:
        raise ArtifactValidationError(f"would produce {sample_count} samples (max {_MAX_SAMPLES})")
    compiled = {state: parse_expression(laws[state]) for state in states}
    values = {state: float(initial.get(state, 0.0)) for state in states}
    times: list[float] = []
    series: dict[str, list[float]] = {state: [] for state in states}

    time = start
    for _ in range(sample_count):
        times.append(round(time, 9))
        for state in states:
            series[state].append(values[state])
        if method == "euler":
            deriv = _derivatives(compiled, states, params, values, time_symbol, time)
            values = {state: values[state] + step * deriv[state] for state in states}
        else:
            k1 = _derivatives(compiled, states, params, values, time_symbol, time)
            mid1 = {s: values[s] + 0.5 * step * k1[s] for s in states}
            k2 = _derivatives(compiled, states, params, mid1, time_symbol, time + 0.5 * step)
            mid2 = {s: values[s] + 0.5 * step * k2[s] for s in states}
            k3 = _derivatives(compiled, states, params, mid2, time_symbol, time + 0.5 * step)
            end_state = {s: values[s] + step * k3[s] for s in states}
            k4 = _derivatives(compiled, states, params, end_state, time_symbol, time + step)
            values = {
                s: values[s] + (step / 6.0) * (k1[s] + 2.0 * k2[s] + 2.0 * k3[s] + k4[s])
                for s in states
            }
        time += step
    return {"time": times, "values": series}
