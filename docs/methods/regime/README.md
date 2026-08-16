# Regime and segmentation methods

`lawsynth-regime` implements deterministic univariate mean-shift segmentation (dynamic-programming PELT objective and best binary split), a Gaussian-mean BOCPD monitor, Viterbi decoding for caller-supplied discrete HMMs, and an index-keyed affine regime law book.

The components are deliberately separate. There is no end-to-end regime-discovery pipeline that fits arbitrary symbolic laws or chooses an HMM from continuous observations.
