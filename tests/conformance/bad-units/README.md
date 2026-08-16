# Invalid unit rejection

This is an executable conformance case. Run it from this directory with:

```sh
python3 run.py
```

A checksum-valid bundle containing an unparsable variable unit must be rejected.

The fixture contract is in `input.json`; the observable acceptance, rejection, or explicitly unsupported outcome is in `expected.json`. The runner builds the archive and invokes the native LawSynth CLI for supported and invalid cases. It does not implement a shadow simulator.
