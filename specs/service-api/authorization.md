# Authorization boundary

The shared API types contain project and run identifiers but no principals,
roles, ACLs, tenancy model, or authorization evaluator. Consequently no local
library operation confers access to a project, artifact, world revision, or run.

A service MUST authorize every read and mutation against its own tenant policy,
including event subscriptions and artifact downloads. It MUST verify project/run
association server-side; identifiers are opaque references, not permissions.
