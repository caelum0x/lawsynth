# CSV ingestion

`lawsynth discover` reads a UTF-8 text file with a header and comma-separated, finite numbers. It does not implement quoted fields, locale-specific decimal separators, missing-value sentinels, or schema inference. Export a clean, numeric table before invoking it.

```sh
lawsynth discover measurements.csv --time time --state x,y \
  --degree 2 --threshold 0.05 --output recovered.lsworld
```

The `--time` name must occur in the header. Every nonblank data row must have the same field count as the header, and the selected time values must strictly increase. State names must be valid LawSynth identifiers; use a preprocessing rename such as `air_temperature` instead of spaces or punctuation.

Run the command on a copy of production data and retain its exact input hash, CLI arguments, and engine version beside the resulting bundle. A CSV with comments, quoted commas, blank cells, `NaN`, or `Infinity` should be parsed and cleaned in a dedicated data tool first, never edited by substring replacement.
