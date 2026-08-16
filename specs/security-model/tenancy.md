# Tenancy

There is no multi-tenant server in the implemented repository. World objects,
bundles, runner envelopes, cancellation tokens, resource limiters, and secrets
are in-process values with no tenant id or access-control check.

Any multi-user deployment must isolate data and execution by authenticated
tenant at the service, storage, queue, and compute layers. Per-run resource
accounting is not tenant isolation, and an HMAC tag does not establish a tenant
identity.
