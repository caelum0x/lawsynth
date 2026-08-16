# LawSynth scalar expression language 0.1

This specification defines the deterministic scalar AST implemented by `lawsynth-expr`. Expressions carry no units or types directly; World IR supplies symbol units and validates dimensions at world construction. Values are IEEE-754 `f64`, but accepted constants and successful evaluation results are finite.

The language supports constants, identifiers, five unary operations, and five binary operations. It has no booleans, vectors, matrices, comparisons, conditionals, random functions, user-defined functions, implicit multiplication, or assignment.
