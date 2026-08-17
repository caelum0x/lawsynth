# LawSynth container images

Canonical, root-context Dockerfiles for every deployable LawSynth service. The
Compose stacks under `deploy/compose/` and the Helm chart under
`deploy/helm/lawsynth/` consume the images produced here.

## Images

| File | Image | Runtime | Port | Entrypoint |
|---|---|---|---:|---|
| `api.Dockerfile` | `ghcr.io/lawsynth/api` | Python 3.12 + gunicorn | 8080 | `lawsynth_api.main:application` |
| `gateway.Dockerfile` | `ghcr.io/lawsynth/gateway` | Python 3.12 + gunicorn | 8081 | admission shim wrapping the API WSGI app |
| `scheduler.Dockerfile` | `ghcr.io/lawsynth/scheduler` | Rust (distroless-style Debian slim) | — | `lawsynth-scheduler` |
| `worker.Dockerfile` | `ghcr.io/lawsynth/worker` | Rust (Debian slim) | — | `lawsynth-worker` |
| `artifact.Dockerfile` | `ghcr.io/lawsynth/artifact` | Rust (Debian slim) | 8082 | `lawsynth-artifact serve` |
| `studio.Dockerfile` | `ghcr.io/lawsynth/studio` | static assets + nginx-unprivileged | 8083 | SPA |
| `development.Dockerfile` | `ghcr.io/lawsynth/development` | Rust + Python + Node toolchain | — | `bash` (dev container / CI base) |

## Conventions

Every runtime image follows the same rules as the repository root `Dockerfile`:

- **Multi-stage build.** A fat builder stage compiles/installs; a minimal
  runtime stage carries only the artifact and its runtime dependencies.
- **Non-root.** Services run as uid/gid `65532` (`lawsynth`); the dev image
  runs as uid/gid `1000` (`dev`); studio runs as the `nginx` user.
- **`--locked` / `--frozen-lockfile` builds** so images are reproducible.
- **HEALTHCHECK** on every image, matching the probe the orchestrators use.
- **No secrets baked in.** Configuration is injected at runtime via
  environment variables (see the Compose `.env.example` files).

## Building

The build context is the **repository root**, not this directory.

```bash
# From the repository root — single image:
docker build -f deploy/docker/images/api.Dockerfile -t ghcr.io/lawsynth/api:0.1.0 .

# All service images at once, via Buildx Bake:
docker buildx bake -f deploy/docker/images/build.hcl

# Multi-arch release build + push:
REGISTRY=ghcr.io/lawsynth VERSION=0.1.0 \
  docker buildx bake -f deploy/docker/images/build.hcl \
  --set *.platform=linux/amd64,linux/arm64 --push release
```

`.dockerignore` in this directory keeps the root context small and prevents
local state (`target/`, `node_modules/`, `.env`, keys) from entering any image.

## Non-goals

These images provide the process and its runtime dependencies only. TLS
termination, secret provisioning, persistent volume ownership, and network
policy are the responsibility of the deployment layer (Compose overlays, the
reverse proxy, Helm, or Kubernetes) — never of the image itself.
