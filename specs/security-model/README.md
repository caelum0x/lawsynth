# Security model

LawSynth is presently an in-process scientific library and CLI. Its code
validates untrusted-looking bundle structure, bounds selected inputs, rejects
invalid expressions and resource requests, and supplies a standalone HMAC
helper. It does not implement identity, authorization, tenant isolation,
network transport, secrets storage, sandboxing, plugin execution, telemetry,
or an artifact service.

Deployers must put authentication, authorization, process isolation, secret
management, rate limiting, and audit retention at their own boundary. The
documents here are precise about current controls and do not turn planned
platform capabilities into claimed protections.
