# Hardware class

The current APIs do not inspect or serialize hardware. Deterministic iteration
order reduces avoidable variability, but floating-point arithmetic can still
differ across CPU architecture, compiler settings, math libraries, and
parallel execution choices.

When results require numerical comparison, capture target architecture, CPU
model or class, operating system, compiler, build profile, thread policy, and
floating-point tolerance. Compare scientifically meaningful numeric tolerances
unless an experiment has demonstrated byte-identical output on its declared
environment.
