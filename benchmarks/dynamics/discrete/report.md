# Discrete logistic map: report contract

The benchmark report is produced by `score.py` as JSON. Its only pass criterion is
observable execution of the declared public LawSynth capability. It does not compare
strings from a synthetic solver or claim exact recovery where the public product lacks
the required representation.

Target process: `x[t+1] = 3.7*x[t]*(1-x[t])`.
