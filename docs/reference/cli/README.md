# LawSynth CLI

`lawsynth` is an offline command-line interface for discovering continuous laws from a numeric CSV file and executing `.lsworld` bundles. It writes machine-readable CSV to standard output for simulation and a short status line for discovery. Operational failures and invalid arguments are written to standard error and return exit status `2`.

The shipped command surface is `inspect`, `discover`, `simulate`, and `simulate-discrete`. There is no configuration-file loader, daemon, web server, plugin command, Studio command, or intervention-only command in this distribution; do not build automation around those names.

Run `lawsynth` with no arguments to obtain the authoritative usage text for the installed binary.
