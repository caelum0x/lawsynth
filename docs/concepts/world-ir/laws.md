# Laws

`ContinuousLaw { target, expression }` defines `d target / dt`; `DiscreteLaw` defines a next value for the target. A target must identify a state variable. The expression IR evaluates scalar `f64` values against the current state, parameters, and inputs.

`dependency_graph()` reports the identifiers read by each law. It is a structural read-set, not a causal graph and not a proof that every syntactic dependency has an identifiable physical effect.

LawSynth does not compile implicit equations, differential-algebraic systems, PDEs, vector fields, or a symbolic unit conversion layer.
