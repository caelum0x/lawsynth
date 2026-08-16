import { migrationPath, migrateDocument } from "../src/migrations.js";

const current = { formatVersion: "0.1.0", id: "current-world" };

/** Current documents need no migration; historical migrations must be registered explicitly. */
export const noMigrationNeeded = migrationPath("0.1.0");
export const migratedCurrentDocument = migrateDocument(current);
