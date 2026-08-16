# Declared assumptions

`AssumptionSet` is an ordered set of declarations: faithfulness, causal sufficiency, and an edge-specific no-unmeasured-confounding declaration. Declaring the same assumption twice has no effect.

`validate_against` requires every edge-specific confounding declaration to name an existing directed graph edge. Otherwise it returns `InvalidParameter("confounding assumption must name a graph edge")`.

Declarations are metadata supplied by the caller. The crate neither tests faithfulness, detects hidden confounders, nor establishes whether a no-confounding assumption is true.
