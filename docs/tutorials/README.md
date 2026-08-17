# Tutorials

Longer, narrative walkthroughs that chain many LawSynth features into one story.
Each is runnable end to end and deterministic/offline. For short, single-task
recipes, see the [cookbook](../cookbook/README.md).

1. [From a messy sensor CSV to a trusted, shareable model](messy-sensor-to-trusted-model.md)
   — profile → prep → discover → explain → validate → backtest → ensemble →
   report → export → register.
2. [A predator–prey ecology walkthrough](predator-prey-ecology.md)
   — generate data, discover with the ecology recipe, read the laws, run
   what-ifs and scenario boards, and export.
3. [Monitoring a running system](monitoring-a-running-system.md)
   — discover a model of "normal", score fresh batches for drift, and decide.

Prerequisites: a built `lawsynth` CLI and the `lawsynth` Python SDK (see
[installation](../getting-started/installation.md)); optionally the
`lawsynth-notebook` package for interactive views. Verify your setup with
`lawsynth doctor`.
