# Core concepts

A **Dataset** is a validated numeric time axis plus named numeric columns.
Time values must be finite and strictly increasing. A **World** contains
validated identifiers, dimensional units, variables, scalar parameters, and
one continuous or discrete transition law per state. An expression is a
finite scalar tree of constants, symbols, and the supported arithmetic and
unary operations.

Discovery profiles input, estimates derivatives, builds deterministic feature
columns, fits sparse coefficients, scores candidates, and creates a World.
Simulation evaluates that World with RK4 for continuous laws or discrete
stepping for discrete laws. A `.lsworld` bundle is the canonical serialized
artifact with integrity checks.

The current bundle format intentionally excludes stochastic laws, delays,
regimes, custom operators, signatures, and causal metadata. Those inputs are
rejected rather than silently simplified.
