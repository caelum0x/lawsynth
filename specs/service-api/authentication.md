# Authentication boundary

There is no authentication implementation, token format, session mechanism, or
identity provider in the local LawSynth distribution. This document therefore
does not prescribe bearer tokens, cookies, mTLS, OAuth, or API keys.

Any service implementation MUST authenticate a caller before accepting mutable
operations or revealing project-scoped data, and MUST bind the resulting
principal to an authorization decision. Authentication headers and failure
responses are transport-specific and must be documented by that service.
