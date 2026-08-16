# Crossover

`crossover_sum(left, right)` produces a simplified sum of two parent expressions. It is deterministic and relies on the expression IR's simplifier for canonical local cleanup.

It is not subtree crossover and does not enforce a population size, depth budget, units, or fitness-based parent choice.
