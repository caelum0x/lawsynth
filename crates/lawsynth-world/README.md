# lawsynth-world

Typed executable World IR for continuous and discrete models. A world combines
named variables, parameters, laws, events, and intervention metadata before a
simulator compiles it.

## Use

```rust
use lawsynth_core::Identifier;
use lawsynth_expr::parse;
use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole, World};

let x = Identifier::new("x")?;
let world = World::new(
    vec![Variable::new(x.clone(), VariableRole::State)],
    vec![Parameter::new(Identifier::new("rate")?, 0.5)],
    vec![ContinuousLaw::new(x, parse("rate * x")?)],
)?;
assert_eq!(world.laws().len(), 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Construction validates unique roles, parameter names, and law targets.
`lawsynth-world` stores the model; numerical integration belongs to
`lawsynth-sim` and discovery belongs to `lawsynth-discovery`.
