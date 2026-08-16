# Health

`GET /health` is intentionally unauthenticated and checks the configured local
database connection and storage root. It returns `ok` only when both probes
succeed; it does not expose secrets, object names, or database configuration.
