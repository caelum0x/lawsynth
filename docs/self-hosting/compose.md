# Container composition boundary

This repository does not ship a Docker Compose file or production container image for the server core. The in-process application has no web-server dependency, so a container definition would need to choose and maintain an HTTP adapter, TLS termination, user identity, volume ownership, health probing, and secret injection. Those are deployment-specific decisions.

If you containerize it for a local trusted environment, mount separate persistent volumes for the SQLite database and object root; run as a non-root user; inject `Settings.tokens` from a secret mechanism rather than an image layer; and bind any adapter to loopback until its network policy is reviewed. Test a stop/start and backup restore against those exact volumes.

Kubernetes manifests, Helm charts, Compose orchestration, managed databases, and cloud object storage are unavailable, not implied future defaults.
