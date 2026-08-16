# Regimes

`lawsynth-regime` analyzes finite scalar signals for mean changes and provides finite-state HMM tools. Its PELT implementation evaluates the exact penalized least-squares recurrence; BOCPD produces online Gaussian change probabilities; Viterbi decodes a validated discrete HMM.

Each algorithm has a narrow statistical interpretation. Keep segmentation and model-selection decisions separate from claims about a physical regime or causal mechanism.
