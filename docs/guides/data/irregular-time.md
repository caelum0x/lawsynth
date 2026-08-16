# Irregular sampling

The time axis may be irregular, but it must be finite and strictly increasing. Finite-difference derivative estimates become sensitive to very small or very large gaps, so inspect interval statistics before discovery. Do not replace timestamps with row numbers unless samples truly share a constant physical interval.

Use the actual times in the CSV or `Dataset`, then compare derivative choices and discovery results on the same held-out interval. If resampling is required, record the interpolation method, target grid, and excluded regions. Resampling is upstream data processing; the engine does not invent a grid or infer an interpolation policy.

For discontinuous measurements, segment the experiment rather than forcing one derivative estimator across an unobserved gap. Regime-aware and missing-observation models are not part of the current discovery API.
