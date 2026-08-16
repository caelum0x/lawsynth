# expression-limits

This executable case loads its declarative input and exercises the public native
LawSynth CLI. It verifies **expression depth exceeds 128** without emulating a server, plugin, or
scheduler. Where that capability is not part of the current build, rejection is
the expected, documented result.

Run from the repository root:

```sh
python3 tests/security/expression-limits/run.py
```
