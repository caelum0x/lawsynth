# causal/lagged executed family case

This benchmark generates a deterministic causal dataset and runs the real
LawSynth `discover --causal` pipeline through the compiled CLI. The CLI emits a
dependency_edges signal (dependency-hypothesis edges), which is scored against a ground-truth-derived
minimum declared in `benchmark.toml`'s `[expect]` table.

Run `python3 run.py` to execute and score the case, `python3 score.py` for
the same scored result, and `python3 generate.py` to write the reproducible
observation CSV. The signal is a real (partial) structural measurement from the
engine, never a fabricated identification, segmentation, or coverage number.
