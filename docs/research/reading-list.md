# Reading list

The implementation is closest to sparse identification of nonlinear dynamics (SINDy), numerical differentiation and smoothing, symbolic-regression search, and numerical ODE simulation. Useful starting points are Brunton, Proctor, and Kutz (2016) on sparse identification; Rudy et al. (2017) on PDE-FIND; Schaeffer (2017) on sparse regression for dynamical systems; and standard numerical-analysis texts for finite differences, splines, Runge--Kutta methods, and stochastic Euler--Maruyama integration.

Read these as methodological context, not as a statement that every algorithm in those papers is implemented here. The supported behavior is defined by the public crate APIs and tests. In particular, current symbolic search is bounded and deterministic, and causal utilities provide graph and predictive-lag primitives rather than a general causal-identification engine.
