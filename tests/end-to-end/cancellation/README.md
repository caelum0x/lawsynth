# Cancellation boundary

The synchronous CLI has no durable run handle to cancel. This case verifies the
unsupported command fails clearly instead of presenting a fake cancellation API.
