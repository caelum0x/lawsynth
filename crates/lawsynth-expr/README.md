# lawsynth-expr

Language-neutral scalar expression IR with a deterministic parser, printer,
evaluator, and local simplification primitives. It is used by world laws and
feature libraries; expression evaluation is pure and accepts values only
through an explicit environment.

## Use

```rust
use lawsynth_core::Identifier;
use lawsynth_expr::{Environment, evaluate, parse, print};

let expression = parse("rate * population - loss")?;
let environment = Environment::from([
    (Identifier::new("rate")?, 0.2),
    (Identifier::new("population")?, 10.0),
    (Identifier::new("loss")?, 1.0),
]);
assert_eq!(evaluate(&expression, &environment)?, 1.0);
assert_eq!(print(&expression), "rate * population - loss");
# Ok::<(), Box<dyn std::error::Error>>(())
```

Parsing rejects malformed syntax and evaluation rejects missing symbols and
non-finite results. The crate represents scalar arithmetic only; dimensional
checking belongs to `lawsynth-units`, while equivalence saturation belongs to
`lawsynth-egraph`.
