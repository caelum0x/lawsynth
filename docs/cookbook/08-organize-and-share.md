# Organize & share your work

**Goal:** keep track of the worlds you discover and the experiments that produced
them — with provenance you can trust — and hand them to others.

LawSynth's shareable unit is the **`.lsworld` bundle**: one portable, content-
addressed file that the CLI, SDK, Studio, and services all operate on. Two local,
deterministic registries organize your work around it:

- **`library`** — a provenance-aware registry of named worlds.
- **`runs`** — content-addressed tracking of `discover` experiments.

Both default to `~/.lawsynth/` and accept `--dir` to point elsewhere.

## The library (named worlds + provenance)

```bash
# register a world under a name, capturing provenance
lawsynth library add prey.lsworld --name predator-prey \
  --tags ecology,demo \
  --from-data prey.csv \
  --config "ecology preset, degree 2" \
  --note "first clean fit"

lawsynth library list
lawsynth library show predator-prey
lawsynth library search ecology            # matches name, tags, description
lawsynth library compare predator-prey other-model --html diff.html
lawsynth library remove predator-prey
```

`add` records a **SHA-256 content hash** of the bundle and — with `--from-data` —
the source data's hash and column set, so you can always tell *which data* and
*which bundle* an entry came from. **Expected shape** of `show`:

```
name:        predator-prey
path:        prey.lsworld
tags:        ecology, demo
description: first clean fit
world hash:  <sha256>
data hash:   <sha256>
data cols:   time,x,y
config:      ecology preset, degree 2
world:       2 state(s), 2 variable(s), 0 parameter(s)
```

`library compare A B` resolves both names to bundle paths and runs the world diff
(text, `--json`, or `--html`). The index is a plain TSV, sorted by name, written
deterministically and never clobbered.

## Experiment tracking (runs)

Add `--track` to any `discover` to record the experiment. The run's **id is
derived from its data hash + configuration** (never a wall clock), so the same
experiment resolves to the same id and re-recording is idempotent.

```bash
# track two experiments that differ only in threshold
lawsynth discover prey.csv --time time --state x,y --output prey.lsworld \
  --preset ecology --track --label baseline

lawsynth discover prey.csv --time time --state x,y --output prey2.lsworld \
  --preset ecology --threshold 0.005 --track --label low-threshold

lawsynth runs list
lawsynth runs show <id>
lawsynth runs compare <id-a> <id-b>
```

**Expected shape** of `runs list`:

```
2 run(s) in ~/.lawsynth/runs
  id            label             degree  thresh        complexity  mse
  <id>          baseline          2       5.000000e-2   <n>         <e>
  <id>          low-threshold     2       5.000000e-3   <n>         <e>
```

`runs compare` diffs the two records' config and result, with signed numeric
deltas on result fields (mse, complexity, …). Records live under
`~/.lawsynth/runs/` (override with `--runs-dir` on `discover`, `--dir` on `runs`).

## From the SDK

The SDK's portable unit is the same `.lsworld` bundle:

```python
import lawsynth

study  = lawsynth.Study.from_csv("prey.csv", time="time", state=["x", "y"])
study.discover()
study.save("prey.lsworld")                          # persist the bundle

# reload later and rebind it to its originating dataset
reloaded = lawsynth.Study.load(
    "prey.lsworld",
    dataset=study.dataset, state=["x", "y"],
)
```

## Share a whole workspace

Bundle every registered world (bundle bytes + provenance) and the runs registry
into one portable, integrity-checked `.lsworkspace` archive — to move machines or
hand a colleague your models:

```sh
lawsynth workspace export team-models.lsworkspace
lawsynth workspace import team-models.lsworkspace --dir ~/.lawsynth   # non-destructive; --force to overwrite
```

Every world is verified against its recorded SHA-256 on import; existing names
are skipped unless `--force`. From Python, `lawsynth.Project` does the same and
reads/writes the **same** `library.tsv` format, so the CLI and SDK share one
workspace:

```python
import lawsynth
project = lawsynth.Project("~/.lawsynth")
project.add("predator-prey", study, tags=("ecology",), note="first clean fit")
project.save()
project.export("team-models.lsworkspace")
# elsewhere:
lawsynth.Project.import_archive("team-models.lsworkspace", "~/.lawsynth")
```

> **Also:** a multi-tenant **projects** API exists on the *service* side (see
> [recipe 10](10-service-and-jupyter.md), `/v1/projects`) for shared, hosted
> workspaces. Locally, `library`/`runs`/`workspace` and the SDK `Project`
> interoperate on the same directory.

## See also

- [Take your model elsewhere](07-export-model.md) — export the registered bundle.
- [Automate a reproducible pipeline](09-reproducible-pipeline.md) — regenerate bundles from config.
