# Exporting trajectories

`lawsynth simulate` writes CSV to standard output. Redirect it to a new file and keep command errors on stderr so a failed run cannot be mistaken for a short trajectory.

```sh
lawsynth simulate model.lsworld --initial x=1 --start 0 --end 1 --step 0.01 \
  > results/run-001.csv
```

The header begins with `time` and then the world state identifiers. Values are emitted in a high-precision scientific representation. Associate the CSV with the source bundle hash, command, engine version, units, and scenario metadata; CSV alone does not encode those facts.

Exporting plots, Parquet, Arrow, database rows, or a web dashboard is outside the CLI. Convert the validated CSV in a downstream tool that owns the target schema and provenance.
