# regime/hmm executed family case

This benchmark generates a deterministic regime dataset and runs the real
LawSynth `discover --regimes` pipeline through the compiled CLI. The CLI emits a
regime_segments signal (regime segmentation), which is scored against a ground-truth-derived
minimum declared in `benchmark.toml`'s `[expect]` table.

Run `python3 run.py` to execute and score the case, `python3 score.py` for
the same scored result, and `python3 generate.py` to write the reproducible
observation CSV. The signal is a real (partial) structural measurement from the
engine, never a fabricated identification, segmentation, or coverage number.
