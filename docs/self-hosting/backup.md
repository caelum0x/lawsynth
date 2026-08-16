# Backup and restore

Quiesce writes before taking a consistent backup. Copy the SQLite database and the complete configured object-root directory as one backup set, then record the application revision and settings that selected their paths. Use filesystem snapshots only if they are coordinated across both locations.

To test restoration, create a new empty instance, restore the database and object root, start the application with a fresh local token, query `/health`, and read a known artifact through the domain API. A successful file copy alone does not verify database-to-object consistency.

The local core does not implement scheduled backups, point-in-time recovery, artifact garbage collection, or cross-region replication. Those belong to host operations or a future storage adapter.
