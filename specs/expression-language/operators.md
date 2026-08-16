# Operators

The AST variants are `Constant(f64)`, `Symbol(Identifier)`, `Unary { operator, operand }`, and `Binary { operator, left, right }`. Unary tags are `Negate`, `Exp`, `Log`, `Sin`, and `Cos`. Binary tags are `Add`, `Subtract`, `Multiply`, `Divide`, and `Power`.

Evaluation has the usual scalar meanings with guarded failures: division rejects exactly zero denominators; `log` requires a strictly positive operand; an absent environment symbol fails; and every operation rejects a non-finite result, including overflow and invalid powers. `sin` and `cos` operate on scalar radians, with dimensional admissibility handled separately.
