# LawSynth Information Diffusion

This is a native Rust batch application for calibrating an independent-cascade
baseline from observed network cascades and comparing a baseline forecast with
one explicit intervention. It is not a frontend and it does not manufacture
sample traction or substitute guessed virality scores for observations.

## Input contract

Inputs are UTF-8 TSV files with exact headers. Every file is limited to 64 MiB;
the model applies additional node, edge, cascade, activation, horizon, and
simulation limits.

| File | Exact header | Meaning |
| --- | --- | --- |
| nodes | `node_id` | Unique graph nodes |
| edges | `source<TAB>target` | Unique directed candidate-transmission paths |
| cascades | `cascade_id<TAB>started_at<TAB>observation_end_step` | Observation windows; `started_at` is `YYYY-MM-DDTHH:MM:SSZ` |
| activations | `cascade_id<TAB>node_id<TAB>step` | At most one activation per node and cascade |
| seeds | `node_id` | Forecast seeds |
| blocked nodes | `node_id` | Optional intervention input |

The command fails closed on malformed rows, unknown references, duplicates,
out-of-window activations, missing exposure observations, bound violations, and
deadline expiry. Hard caps are 5,000 nodes, 50,000 edges, 2,000 cascades,
500,000 activations, 10,000 observation steps, 2,000,000 calibration
observations, a 180-step forecast, 5,000 simulations, and 600 seconds of runtime.

## Run

From the LawSynth workspace root:

```console
cargo run -p lawsynth-information-diffusion -- \
  --nodes /absolute/path/nodes.tsv \
  --edges /absolute/path/edges.tsv \
  --cascades /absolute/path/cascades.tsv \
  --activations /absolute/path/activations.tsv \
  --seeds /absolute/path/seeds.tsv \
  --blocked-nodes /absolute/path/blocked-nodes.tsv \
  --horizon 30 \
  --simulations 1000 \
  --seed 42 \
  --transmission-multiplier 0.75 \
  --max-runtime-ms 30000 \
  --output /absolute/path/report.json
```

Existing reports are preserved unless `--overwrite` is supplied. A report is
written through a same-directory temporary file, synced, and installed
atomically; an input file cannot be selected as the output. On Unix, new reports
use owner-only permissions. The JSON contains the normalized data digest,
calibration and chronological holdout metrics, seeded forecast bands, model
limitations, and a content-addressed receipt digest.

## Interpretation boundary

Candidate edges are observational paths, not proof of causality. The fitted
probability is global and each candidate edge gets one attempt when its source
activates. Absence is interpreted only within each explicit observation window,
and forecast bands describe simulation-process variation.
Parameter confidence is reported separately. Fewer than three cascades, or a
holdout without measurable exposure, produces an `unmeasured` backtest with a
reason instead of a fabricated score.
