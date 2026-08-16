# Identification boundary

The crate does not implement back-door, front-door, do-calculus, instrumental-variable, or potential-outcome identification. It therefore never claims an average treatment effect, mediation effect, or counterfactual estimate.

`CausalGraph` validates DAG topology and `AssumptionSet` validates only that certain declarations reference actual edges. Neither operation proves that a supplied graph is data-generating or that an estimand is identified.

Use these primitives to make preconditions explicit, then delegate identification and estimation to an implementation whose scope includes the required estimand and data model.
