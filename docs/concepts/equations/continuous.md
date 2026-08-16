# Continuous laws

A `ContinuousLaw` declares the derivative of one state. `lawsynth-sim::simulate` integrates all state laws with classical fourth-order Runge-Kutta and records the initial condition plus accepted steps. Expressions read the current stage state, parameters, and controls.

The integrator validates request values and checks calculated state values for finiteness. It uses a fixed requested step, shortened at the final interval or at a scheduled input/parameter change.

The path does not provide adaptive error control, stiffness detection, dense output, sensitivity equations, or root-finding events.
