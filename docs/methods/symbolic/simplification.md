# Simplification

`simplify_candidate` delegates to the expression IR simplifier. The e-graph normalizer recursively applies safe local simplification and canonical operand ordering for addition and multiplication.

The system avoids algebraic transformations whose validity would depend on domains, such as cancellation across a potentially zero denominator. It is not a computer-algebra system with assumptions, integration, or symbolic differentiation.
