# Regime timeline boundary

The current native discovery and simulation interfaces do not implement regime segmentation, changepoint inference, hybrid switching, or a regime-aware timeline. A Studio integration must not present these as completed analytical results.

If an external analysis identifies known experiment phases, it may display those phases as provenance annotations alongside trajectories. Keep the phase source, time unit, and decision method explicit, and distinguish annotations from engine-derived events.

State-dependent switches and reset maps require a separate hybrid-dynamics implementation and validation suite. Until then, use only implemented scheduled parameter/input changes for known exogenous times.
