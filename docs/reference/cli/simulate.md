# `lawsynth simulate` and `simulate-discrete`

Continuous simulation:

```
lawsynth simulate model.lsworld --initial x=1 --start 0 --end 10 --step 0.01
```

Discrete simulation uses `lawsynth simulate-discrete model.lsworld --initial x=1 --steps 100 [--start 0]`. Both commands accept repeated `--initial NAME=VALUE`, `--parameter NAME=VALUE`, `--input NAME=VALUE`, `--parameter-at TIME:NAME=VALUE`, and `--input-at TIME:NAME=VALUE`. Output is CSV with a `time` header followed by state IDs; numbers are emitted in scientific notation.

Continuous and discrete world semantics are not interchangeable: the command rejects a bundle with the wrong time model. The current executable supports the compiled deterministic solvers only. Stochastic paths, adaptive solver selection, event roots, hybrid modes, delays, and streamed output are outside this CLI contract.
