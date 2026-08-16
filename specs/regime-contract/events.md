# Event boundary

The crate returns segment boundaries, binary split candidates, BOCPD change probabilities, and decoded discrete states. These are computed outputs, not scheduled event objects and not a live event stream.

`bocpd` emits one `BocpdPoint` per input observation containing its index, posterior change probability under the supplied scalar Gaussian model, and the most likely run length. It requires a non-empty finite series, `0 < hazard <= 1`, positive finite observation variance, and positive prior precision.

No debounce policy, event delivery, timestamp conversion, trigger action, or causal attribution is implemented.
