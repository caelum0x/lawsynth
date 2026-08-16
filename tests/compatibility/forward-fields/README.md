# forward-fields

This executable case loads its declarative input and exercises the public native
LawSynth CLI. It verifies **unsupported manifest** without emulating a server, plugin, or
scheduler. Where that capability is not part of the current build, rejection is
the expected, documented result.

Run from the repository root:

```sh
python3 tests/compatibility/forward-fields/run.py
```
