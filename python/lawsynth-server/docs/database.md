# Database

`Database` is a small SQLite transaction wrapper for local metadata adapters.
It enables foreign keys and uses `BEGIN IMMEDIATE`, rolling back on every
exception. Production Postgres schemas and migrations belong to a dedicated
database adapter; this core never claims SQLite is a distributed scheduler.
