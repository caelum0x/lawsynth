# Assumptions

`AssumptionSet` records causal assumptions such as causal sufficiency, faithfulness, and temporal order. `validate_against` checks only structural compatibility with a supplied graph. `DependencyAssumptions` in discovery separately allows or forbids candidate directed dependencies.

Record why each assumption holds for the study design, measurement process, and population. A successful validation means the graph meets the software’s stated preconditions; it does not verify those scientific premises.
