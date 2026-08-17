"""License policy model, aligned with the repository ``deny.toml`` allowlist.

The policy understands a small, practical slice of SPDX expression syntax:
``AND``, ``OR``, ``WITH`` exceptions, and parenthesised groups. An expression is
allowed when every required license in it is on the allowlist. ``OR`` means any
one branch being allowed is sufficient; ``AND`` requires all operands.
"""

from __future__ import annotations

import re
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

# Mirrors deny.toml [licenses].allow so a single change keeps Rust and this
# tool in agreement.
DEFAULT_ALLOW = (
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "MIT",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
    "Zlib",
)

_TOKEN = re.compile(r"\s+(AND|OR)\s+", re.IGNORECASE)


@dataclass(frozen=True)
class Policy:
    """An allowlist of SPDX identifiers plus explicit per-license exceptions."""

    allow: frozenset[str] = field(default_factory=lambda: frozenset(DEFAULT_ALLOW))

    def permits_license(self, spdx: str) -> bool:
        """Return whether a single (non-compound) license token is allowed."""
        return spdx.strip() in self.allow

    def permits_expression(self, expression: str | None) -> bool:
        """Evaluate a possibly-compound SPDX expression against the allowlist."""
        if expression is None or not expression.strip():
            return False
        return _evaluate(expression.strip(), self)


def _evaluate(expression: str, policy: Policy) -> bool:
    expression = expression.strip()
    if expression.startswith("(") and expression.endswith(")") and _balanced(expression[1:-1]):
        expression = expression[1:-1].strip()

    parts = _split_top_level(expression)
    if parts is not None:
        operator, operands = parts
        results = [_evaluate(operand, policy) for operand in operands]
        return any(results) if operator == "OR" else all(results)

    # Atom: possibly "<license> WITH <exception>".
    return policy.permits_license(expression)


def _split_top_level(expression: str) -> tuple[str, list[str]] | None:
    """Split on the lowest-precedence top-level operator (OR before AND)."""
    for operator in ("OR", "AND"):
        operands = _split_operator(expression, operator)
        if len(operands) > 1:
            return operator, operands
    return None


def _split_operator(expression: str, operator: str) -> list[str]:
    operands: list[str] = []
    depth = 0
    current = []
    tokens = re.split(r"(\s+)", expression)
    index = 0
    while index < len(tokens):
        token = tokens[index]
        depth += token.count("(") - token.count(")")
        if depth == 0 and token.upper() == operator:
            operands.append("".join(current).strip())
            current = []
        else:
            current.append(token)
        index += 1
    operands.append("".join(current).strip())
    return [operand for operand in operands if operand]


def _balanced(text: str) -> bool:
    depth = 0
    for char in text:
        depth += (char == "(") - (char == ")")
        if depth < 0:
            return False
    return depth == 0


def load_policy(deny_toml: Path) -> Policy:
    """Build a policy from a cargo-deny ``deny.toml`` (the ``[licenses].allow`` list)."""
    data = tomllib.loads(deny_toml.read_text(encoding="utf-8"))
    allow = data.get("licenses", {}).get("allow", list(DEFAULT_ALLOW))
    return Policy(allow=frozenset(allow))
