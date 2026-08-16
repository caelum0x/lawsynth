# Streaming boundary

ApiEvent has a sequence number, occurrence timestamp in milliseconds, project
identifier, optional run identifier, event kind, and bounded UTF-8 payload.
Kinds are RunQueued, RunStarted, Progress, RunSucceeded, RunFailed,
RunCancelled, and ArtifactCreated. validate_event_stream requires a strictly
increasing sequence and preserves project scope.

This is an event value contract only. No SSE, WebSocket, long-poll, broker,
acknowledgement, replay, retention, or ordering guarantee across processes is
implemented. A service exposing events MUST define those delivery semantics.
