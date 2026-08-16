# Continuous ODE integration

`simulate` compiles continuous laws once, checks that every declared state has one finite initial value, then advances with classical RK4. It shortens the final step to end exactly at the requested endpoint and splits an integration interval at scheduled parameter/input changes.

The method is fixed-step RK4, not an adaptive solver. There are no embedded error estimates, stiffness detection, dense output, root finding, or automatic step control.
