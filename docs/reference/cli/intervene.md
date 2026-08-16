# Scheduled interventions

Interventions are supplied to simulation, not through a standalone `intervene` command. Use `--parameter-at TIME:NAME=VALUE` or `--input-at TIME:NAME=VALUE`; `TIME` and `VALUE` must be finite and `NAME` must be a valid LawSynth identifier.

The assignment is a scheduled constant override in the simulation request. Event-triggered interventions, optimization of interventions, counterfactual search, and causal effect estimation are not implemented by this CLI.
