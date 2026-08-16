# Quickstart: recover and simulate a law

Create a numeric CSV with a header, a strictly increasing finite time column,
and one or more finite numeric state columns. For example `observations.csv`:

```csv
time,x
0,1
0.1,0.9802
0.2,0.9608
```

Recover a continuous scalar World and inspect its bundle:

```sh
cargo run -p lawsynth-cli -- discover observations.csv \
  --time time --state x --output decay.lsworld
cargo run -p lawsynth-cli -- inspect decay.lsworld
cargo run -p lawsynth-cli -- simulate decay.lsworld \
  --initial x=1 --start 0 --end 1 --step 0.1
```

Discovery is deterministic for identical input and options. Its output is a
sparse candidate selected from the implemented scalar feature library; it is
not evidence that an inferred relation is causal or scientifically valid.
