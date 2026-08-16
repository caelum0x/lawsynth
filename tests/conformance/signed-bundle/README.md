# Signed bundle capability boundary

This is an executable conformance case. Run it from this directory with:

```sh
python3 run.py
```

The bundle crate can compute and verify HMAC signatures as an explicit API, but signature material is not part of the .lsworld archive format or CLI.

The fixture contract is in `input.json`; the observable acceptance, rejection, or explicitly unsupported outcome is in `expected.json`. The runner builds the archive and invokes the native LawSynth CLI for supported and invalid cases. It does not implement a shadow simulator.
