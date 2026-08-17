"""Pure-``ast`` symbolic simplification for World law expressions.

Native world expressions are ordinary arithmetic strings — sums of products of
identifiers and numeric constants, e.g. ``((1.04*x)+(-0.39*(x*y)))``. This
module rewrites such an expression into a canonical, minimal-complexity but
*mathematically equivalent* form using only the standard-library :mod:`ast`
module. Nothing is ever evaluated: we parse to a syntax tree, flatten it into a
polynomial of ``(coefficient, monomial)`` terms, fold constants, collapse
identities (``x+0``, ``x*1``, ``x*0``), combine like additive terms, normalise
sign, and re-emit a deterministic string.

Any sub-expression we do not recognise as a polynomial building block (a call
like ``sin(x)``, a division by a non-constant, …) is treated as an *opaque
atom* keyed by its source text: it still participates in like-term combination
and constant folding as a commuting real value, so equivalence is preserved
without the simplifier needing to understand it. If an expression cannot be
parsed at all, it is returned unchanged — we never fabricate a reduction.
"""

from __future__ import annotations

import ast

__all__ = ["simplify_expression", "node_count"]


# A monomial is a sorted tuple of atom keys (identifiers or opaque source
# strings). A term is (coefficient, monomial). An expression flattens to a list
# of terms summed together.
_Monomial = tuple[str, ...]


def node_count(expression: str) -> int:
    """Number of AST nodes in ``expression`` — a proxy for structural complexity."""
    try:
        tree = ast.parse(expression, mode="eval")
    except SyntaxError:
        return 0
    return sum(1 for _ in ast.walk(tree))


_Term = tuple[float, _Monomial]


def _negate(terms: list[_Term]) -> list[_Term]:
    return [(-coeff, monomial) for coeff, monomial in terms]


def _multiply(left: list[_Term], right: list[_Term]) -> list[_Term]:
    """Distribute a product of two sums (the Cartesian product of their terms)."""
    product: list[_Term] = []
    for lc, lm in left:
        for rc, rm in right:
            product.append((lc * rc, tuple(sorted(lm + rm))))
    return product


def _expand(node: ast.AST) -> list[_Term]:
    """Fully expand an expression into a distributed list of polynomial terms.

    Multiplication is distributed over addition, so ``k*(a+b)`` becomes
    ``k*a + k*b``. Every step preserves value: it is exact real arithmetic over
    a commutative ring (subject only to floating-point round-off).
    """
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
        return _expand(node.left) + _expand(node.right)
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Sub):
        return _expand(node.left) + _negate(_expand(node.right))
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.UAdd):
        return _expand(node.operand)
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
        return _negate(_expand(node.operand))
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Mult):
        return _multiply(_expand(node.left), _expand(node.right))
    if _is_constant(node):
        return [(float(node.value), ())]
    if isinstance(node, ast.Name):
        return [(1.0, (node.id,))]
    if (
        isinstance(node, ast.BinOp)
        and isinstance(node.op, ast.Pow)
        and isinstance(node.right, ast.Constant)
        and isinstance(node.right.value, int)
        and not isinstance(node.right.value, bool)
        and node.right.value >= 0
    ):
        result: list[_Term] = [(1.0, ())]
        base = _expand(node.left)
        for _ in range(node.right.value):
            result = _multiply(result, base)
        return result
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div) and _is_constant(node.right):
        divisor = float(node.right.value)
        return [(coeff / divisor, monomial) for coeff, monomial in _expand(node.left)]
    # Anything we do not model (a call, a division by a variable, …) becomes an
    # opaque atom keyed by its source text. Real multiplication is commutative,
    # so this preserves equivalence without understanding the sub-expression.
    return [(1.0, (ast.unparse(node),))]


def _is_constant(node: ast.AST) -> bool:
    return isinstance(node, ast.Constant) and isinstance(node.value, (int, float)) and not isinstance(node.value, bool)


def _combine(terms: list[_Term]) -> list[_Term]:
    """Sum coefficients of like monomials and drop zero terms."""
    combined: dict[_Monomial, float] = {}
    for coeff, monomial in terms:
        combined[monomial] = combined.get(monomial, 0.0) + coeff
    result = [(coeff, monomial) for monomial, coeff in combined.items() if coeff != 0.0]
    # Deterministic ordering: higher-degree monomials first, then lexical.
    result.sort(key=lambda item: (-len(item[1]), item[1]))
    return result


def _format_number(value: float) -> str:
    """Compact but exact-round-tripping numeric literal."""
    if value == int(value) and abs(value) < 1e16:
        return repr(int(value))
    return repr(value)


def _wrap_atom(atom: str) -> str:
    return atom if atom.isidentifier() else f"({atom})"


def _render(terms: list[tuple[float, _Monomial]]) -> str:
    if not terms:
        return "0"
    parts: list[str] = []
    for index, (coeff, monomial) in enumerate(terms):
        magnitude = abs(coeff)
        negative = coeff < 0
        if not monomial:  # pure constant term
            piece = _format_number(magnitude)
        else:
            product = "*".join(_wrap_atom(atom) for atom in monomial)
            piece = product if magnitude == 1.0 else f"({_format_number(magnitude)}*{product})"
        if index == 0:
            parts.append(f"-{piece}" if negative else piece)
        else:
            parts.append(f"- {piece}" if negative else f"+ {piece}")
    return " ".join(parts)


def simplify_expression(expression: str) -> str:
    """Return a canonical, equivalent, minimal-complexity form of ``expression``.

    Falls back to the original text if it cannot be parsed as an expression.
    """
    try:
        tree = ast.parse(expression, mode="eval").body
    except SyntaxError:
        return expression
    terms = _combine(_expand(tree))
    return _render(terms)
