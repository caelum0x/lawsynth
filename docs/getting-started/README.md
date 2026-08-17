# Getting started

LawSynth turns time-series observations into **executable mathematical worlds**:
interpretable law systems you can read, simulate, forecast, compare, and share.
The whole product is one loop, and every step is deterministic and offline:

```
observe (CSV) → discover (laws) → understand (explain) → use (simulate /
forecast / intervene) → compare → share (report / .lsworld bundle) → organize
```

A discovery is a portable `.lsworld` bundle; everything downstream — CLI, Python
SDK, Studio, and the HTTP services — operates on that same artifact.

## Read next

1. **[installation](installation.md)** — build the `lawsynth` CLI and the Python SDK.
2. **[quickstart](quickstart.md)** — run the core loop end to end.
3. **[concepts](concepts.md)** — datasets, worlds, laws, and bundles.
4. **[your first world](first-world.md)** — discover, inspect, and reuse a bundle.
5. Pick a surface: **[CLI](cli.md)**, **[Python](python.md)**, or **[Studio](studio.md)**.

Then go deeper with task-oriented, copy-pasteable material:

6. **[Cookbook](../cookbook/README.md)** — short recipes for each task (discover,
   clean, forecast, validate, monitor, export, organize, pipeline, service,
   Jupyter), each with a CLI recipe *and* its Python-SDK equivalent.
7. **[Tutorials](../tutorials/README.md)** — longer end-to-end walkthroughs that
   chain many features into one narrative (messy sensor → trusted model,
   predator–prey ecology, monitoring a running system).

The checked-in [examples](examples.md) and the workspace tests are executable and
are the source of truth for supported inputs. Discovery finds a sparse fit from the
implemented feature library — it is not evidence that an inferred relation is causal
or that extrapolation beyond the observed window is valid.
