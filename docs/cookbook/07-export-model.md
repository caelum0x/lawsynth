# Take your model elsewhere

**Goal:** turn a discovered `.lsworld` into a standalone, dependency-free artifact
that runs anywhere — no LawSynth required at the destination.

`export` supports six formats. Each embeds the world's laws (coefficients inlined,
any template parameters written in), an RK4 integrator, and a runnable demo where
applicable.

| `--format` | Aliases | What you get |
|-----------|---------|--------------|
| `python` | `py` | dependency-free module: `derivatives(t, state, params)` + `simulate(...)` (RK4) + `__main__` demo |
| `c` | | dependency-free C source: `lawsynth_derivatives`, RK4, and a `main()`. Compile: `cc -O2 model.c -lm -o model` |
| `onnx` | `graph` | a LawSynth computation-graph JSON (ONNX-style op DAG; one output per state derivative) — **not** a binary `.onnx` |
| `matlab` | `octave`, `m` | Octave/MATLAB `.m`: `lawsynth_derivatives(t, state)` + `lawsynth_simulate(...)` + demo |
| `latex` | `tex` | the law system as an `align*` block of `\dot{x} = …` equations |
| `json` | | documented JSON: variables, parameters, and per-law `equation`/`latex`/`python`/`reads` |

## CLI

```bash
# to stdout
lawsynth export prey.lsworld --format latex

# to files
lawsynth export prey.lsworld --format python --output prey.py
lawsynth export prey.lsworld --format c      --output prey.c
lawsynth export prey.lsworld --format onnx   --output prey.graph.json
lawsynth export prey.lsworld --format matlab --output prey.m
lawsynth export prey.lsworld --format json   --output prey.json
```

**Expected shape** — with `--output`, a confirmation prints; without it, the
artifact is written to stdout:

```
wrote prey.py (<bytes> bytes)
```

Run or compile the result with no LawSynth dependency:

```bash
python prey.py                 # prints the final state after the demo integration
cc -O2 prey.c -lm -o prey && ./prey
octave prey.m
```

The generated Python exposes exactly:

```python
# prey.py (generated)
PARAMS = {...}                 # inlined parameters (empty for a pure discovered world)
STATE_VARS = ["x", "y"]
def derivatives(t, state, params): ...   # d(state)/dt as a dict
def simulate(initial, t0, t1, dt, params=None): ...  # RK4 -> (times, traj)
```

## Python SDK

There is no separate SDK export call for these six formats — the **`.lsworld`
bundle is the portable artifact**, and the CLI `export` reads it. From the SDK,
persist the bundle and hand it to `export`:

```python
import lawsynth, subprocess

study  = lawsynth.Study.from_csv("prey.csv", time="time", state=["x", "y"])
study.discover()
study.save("prey.lsworld")                 # portable bundle

subprocess.run(["lawsynth", "export", "prey.lsworld",
                "--format", "python", "--output", "prey.py"], check=True)
```

The SDK does offer a **self-contained HTML report** directly
(`DiscoveryResult.report("out.html")` / `Study.report(...)`) — that's the
share-with-a-human artifact, distinct from the run-anywhere code exports above.

## Notes

- Exports are deterministic: the same world always yields byte-identical output.
- The `onnx` format is an **honest, labeled** computation-graph JSON (it says so
  in the file), not a binary `.onnx` — one graph output per state derivative.
- Non-state inputs (if any) default to `0.0` in the generated code, flagged
  `edit as needed`.

## See also

- [Discover from a CSV](01-discover-from-csv.md) to produce the `.lsworld`.
- [Organize & share your work](08-organize-and-share.md) to register and track bundles.
