# Upgrade procedure

Treat upgrades as data changes even when the Python package has no migration framework. Back up the SQLite database and object root, stop writers, install the new package in a separate environment, run its test suite, then start it against a copy of production-local data and exercise health, repository reads, artifact reads, and idempotent write replay before switching over.

Keep the previous environment and backup until the upgraded instance has passed those checks. If a newer package changes a persisted repository schema, it must ship an explicit migration; no automatic migration system exists in this local core.

Do not use `lawsynth serve` as an upgrade target: the Rust CLI has no daemon mode.
