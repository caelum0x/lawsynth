# lawsynth-cli

The command implementation shared by the `lawsynth` executable and integration
tests. It imports numeric CSV files, runs discovery, writes world bundles,
inspects bundles, and simulates continuous or discrete worlds.

## Use

```sh
lawsynth discover observations.csv --time t --state x,y --output recovered.lsworld
lawsynth inspect recovered.lsworld
lawsynth simulate recovered.lsworld --start 0 --end 10 --step 0.01 --set x=1
```

Discovery flags expose derivative choice, feature families, sparse solver,
bootstrap, and symbolic search bounds. The current `serve` command intentionally
returns a typed “unavailable” error: no server is started by this crate.
CSV input must be rectangular, numeric, and have a strictly increasing time
column; use `lawsynth-data` directly for supported native Parquet subset input.
