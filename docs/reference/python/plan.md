# Discovery plans

`DiscoveryPlan(states)` captures the fixed native pipeline order: `validate`, `preprocess`, `profile`, `differentiate`, `generate_features`, `fit_laws`, `score`, `finalize`. State names must be distinct Python identifiers, and custom stage ordering is rejected.

The object documents execution intent; it does not schedule, persist, resume, distribute, or execute discovery. Use `lawsynth.discover.discover` to run the compiled pipeline.
