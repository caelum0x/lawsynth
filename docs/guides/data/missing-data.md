# Missing observations

Input values must be finite. Blank CSV cells, `NaN`, `Infinity`, and columns of different lengths are rejected. This is a safety boundary: treating a missing measurement as zero creates a false dynamical signal.

Resolve missingness before calling LawSynth. Depending on the experiment, that may mean restricting analysis to complete intervals, combining independently measured replicates, or using a domain-reviewed imputation model. Keep a mask of original missing values and quantify how the chosen policy changes results.

The current engine does not expose a missing-data likelihood, automatic gap bridging, or confidence claims based on imputed values. Use sensitivity runs with alternate justified preprocessing choices rather than presenting a single imputed fit as measured fact.
