# Symbolic search and rewriting

`lawsynth-symbolic` deterministically enumerates a bounded grammar of scalar expressions, applies the expression IR simplifier, supports simple crossover/mutation utilities, affine constant calibration, and a loss/complexity Pareto filter. `lawsynth-egraph` provides bounded local algebraic normalization and extraction.

These tools search syntax within explicit limits. They do not implement probabilistic genetic programming, arbitrary rewrite saturation, or proof of physical equivalence.
