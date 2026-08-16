# Polars conversion

Polars is not a LawSynth dependency. Use it to make selection and ordering explicit, then pass ordinary lists to the Python SDK or write numeric CSV for the CLI.

```python
clean = frame.select("time", "x", "y").sort("time").drop_nulls()
dataset = Dataset.from_columns(
    clean["time"].to_list(),
    {name: clean[name].to_list() for name in ("x", "y")},
)
```

`drop_nulls()` is a policy decision, not a universal remedy: it changes sample spacing and may bias the fitted dynamics. Prefer a documented observation selection or an independently justified imputation procedure. Check that the final time list is strictly increasing and that values are finite; those are the engine's hard input requirements.
