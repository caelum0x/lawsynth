# regime/event-driven capability contract

This executable benchmark generates a deterministic regime dataset and verifies the current LawSynth capability boundary. It does not report an inferred causal effect, regime assignment, or calibrated interval: that capability is not part of the implemented engine.

Run `python3 run.py` to exercise the boundary and `python3 score.py` to validate the recorded contract. `generate.py` writes the reproducible oracle dataset to the system temporary directory without polluting the repository.
