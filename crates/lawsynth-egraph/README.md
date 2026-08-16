# lawsynth-egraph

Bounded equivalence rewriting for `lawsynth-expr` scalar expressions. It makes
canonicalization and algebraic alternatives explicit without allowing an
unbounded rewrite search to consume a discovery run.

## Use

```rust
use lawsynth_egraph::{EquivalenceGraph, RewriteConfig, normalize};
use lawsynth_expr::parse;

let expression = parse("b + a + 0")?;
let canonical = normalize(expression);
let graph = EquivalenceGraph::new(canonical, RewriteConfig::default())?;
assert!(graph.class_count() >= 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`RewriteLimits` bounds nodes, classes, iterations, and wall-clock work.
`extract_lowest_cost` selects an expression using the deterministic structural
cost model. This crate does not prove semantic equality over floating-point
evaluation; its proofs describe applications of its finite rewrite rules.
