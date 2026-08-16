# lawsynth-symbolic

Bounded, deterministic scalar symbolic search built on the common expression
IR. It provides terminal grammars, finite enumeration, population management,
symbol replacement, additive crossover, affine calibration, simplification,
and a loss-versus-complexity Pareto frontier.

## Use

```rust
use lawsynth_core::Identifier;
use lawsynth_symbolic::{Grammar, SymbolicConfig, enumerate};

let grammar = Grammar::scalar([Identifier::new("x")?]);
let candidates = enumerate(&grammar, &SymbolicConfig::default());
assert!(!candidates.is_empty());
# Ok::<(), lawsynth_core::IdentifierError>(())
```

Search is explicitly bounded by `SymbolicConfig`; it does not invoke a language
model or claim global optimality. Fit and validate candidate expressions against
held-out data before promoting them into a world.
