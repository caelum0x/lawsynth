# Resampled ensembles

Stability selection draws rows with replacement, fits STLSQ for each replicate, and counts how often each coefficient survives the configured magnitude threshold. The deterministic RNG is seeded through `lawsynth_core::Seed`, so identical inputs and configuration reproduce frequencies.

These counts describe this resampling procedure only. They are not p-values, calibrated inclusion probabilities, or protection against dependent time-series samples.
