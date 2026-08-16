# Market microstructure

This is a reproducible LawSynth example. It generates observations from the
model stated in [config.toml](config.toml), performs finite-difference
polynomial-library discovery where supported, and simulates the declared
world. All generated artifacts are written below `output/` and are ignored by
version control.

```bash
python generate.py
python discover.py
python simulate.py
python -m pytest test_example.py
```

The data are synthetic and deterministic; they are suitable for exercising the
pipeline, not for claiming causal identification from observational data.

The included standard-library baseline is intentionally transparent. If the
native `lawsynth` extension is installed, discovery invokes it as well and
records its actual outcome in `output/discovery.json`.
