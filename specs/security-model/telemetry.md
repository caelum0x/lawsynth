# Telemetry

No telemetry exporter, tracing SDK integration, metrics backend, analytics
client, or remote logging service is implemented by the current crates. The
engine therefore does not transmit dataset contents, equations, prompts, or
identifiers as telemetry because it does not transmit telemetry at all.

Applications adding observability must make collection opt-in, redact inputs
and artifact contents by default, minimize identifiers, secure transport and
storage, and define retention and deletion policies. `ProgressEvent` messages
are caller-controlled and must not be exported blindly.
