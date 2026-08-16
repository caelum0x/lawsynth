# Serving boundary

This distribution is not a server. The internal `serve` entry point returns an explicit unsupported-operation error and `lawsynth serve` is not part of the command parser. No HTTP API, authentication layer, multi-tenant queue, or background daemon is compiled into the binary.
