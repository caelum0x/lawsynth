# Database

Set `Settings.database_url` to `:memory:` for ephemeral tests or `sqlite:///absolute-or-relative-path.db` for a local persistent database. Other URLs raise an error. The database wrapper enables foreign keys and encloses repository mutations in `BEGIN IMMEDIATE` transactions, providing a single-writer local consistency boundary.

Back up the SQLite database together with the object root. Neither one alone reconstructs a complete service state: metadata refers to artifact hashes and artifacts have no independent tenant or retention metadata. Test restoration into a separate directory before relying on a backup.

PostgreSQL, migrations, replicas, connection pooling, encryption at rest, and cross-process leader election are not implemented. A production database adapter needs its own migration and operational design.
