# Compatibility

Frame protocol version 1 is exact-match only. A decoder rejects every other
version, so a plugin and host MUST negotiate or configure version 1 before
exchanging frames. Reserved frame byte zero MUST remain zero.

Manifest compatibility is intentionally strict: unknown keys and capabilities
reject rather than being ignored. A future protocol version needs a new decoder
and explicit compatibility rule; existing implementations MUST NOT silently
accept future data.
