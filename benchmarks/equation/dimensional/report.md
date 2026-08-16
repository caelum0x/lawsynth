# Dimension-aware acceleration: report contract

The benchmark report is produced by `score.py` as JSON. Its only pass criterion is
observable execution of the declared public LawSynth capability. It does not compare
strings from a synthetic solver or claim exact recovery where the public product lacks
the required representation.

Target process: `position = 0.5*a*t^2; velocity = a*t, a = 9.81 m/s^2`.
