# tenant-isolation

This executable case loads its declarative input and exercises the public native
LawSynth CLI. It verifies **usage:** without emulating a server, plugin, or
scheduler. Where that capability is not part of the current build, rejection is
the expected, documented result.

Run from the repository root:

```sh
python3 tests/security/tenant-isolation/run.py
```
