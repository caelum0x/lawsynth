# Worlds

A World is an executable continuous-time system. Build one from state variables,
optional controls and parameters, and exactly one expression per state. The
native engine validates identifiers and evaluates all updates deterministically.

```python
from lawsynth.equation import Equation
from lawsynth.variable import Variable
from lawsynth.world import build_world
world = build_world((Variable("x"),), {"rate": -1.0}, (Equation("x", "rate * x"),))
```
