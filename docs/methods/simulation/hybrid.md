# Hybrid boundary

The implemented hybrid support is deterministic segmentation at known finite timestamps. Duplicate times coalesce to avoid zero-length integration segments.

There is no guard evaluation, event localization, reset map, mode machine, hysteresis handling, or Zeno-behavior protection. Do not describe scheduled parameter changes as general hybrid simulation.
