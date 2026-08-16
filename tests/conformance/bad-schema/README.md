# Malformed archive rejection

This is an executable conformance case. Run it from this directory with:

```sh
python3 run.py
```

Malformed archive bytes must be rejected before world decoding.

The fixture contract is in `input.json`; the observable acceptance, rejection, or explicitly unsupported outcome is in `expected.json`. The runner builds the archive and invokes the native LawSynth CLI for supported and invalid cases. It does not implement a shadow simulator.
