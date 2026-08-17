# syntax=docker/dockerfile:1.7
#
# LawSynth Studio image.
#
# Studio is the React + TypeScript + WASM front end. It is a static asset
# bundle built from the pnpm workspace and served by an unprivileged nginx.
# Studio talks to the gateway over HTTP/SSE; there is no server-side runtime.
#
# Build from the repository root:
#   docker build -f deploy/docker/images/studio.Dockerfile -t ghcr.io/lawsynth/studio:0.1.0 .

# ---- builder: pnpm workspace build -----------------------------------------
FROM node:22-bookworm-slim AS build
ENV PNPM_HOME=/pnpm \
    PATH=/pnpm:$PATH \
    CI=1
RUN corepack enable

WORKDIR /workspace
COPY . .

# Install the whole workspace (studio depends on several workspace packages),
# then build only studio and its transitive workspace dependencies.
RUN --mount=type=cache,target=/pnpm/store \
    pnpm install --frozen-lockfile \
    && pnpm --filter @lawsynth/studio... run build

# ---- runtime: static assets under unprivileged nginx -----------------------
FROM nginxinc/nginx-unprivileged:1.27-alpine AS runtime

# SPA fallback + long-cache for hashed assets, no-cache for the entrypoint.
COPY --chown=nginx:nginx <<'NGINX' /etc/nginx/conf.d/default.conf
server {
    listen 8083;
    server_name _;
    root /usr/share/nginx/html;
    index index.html;

    location = /healthz {
        add_header Content-Type text/plain;
        return 200 'ok';
    }

    location /assets/ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    location / {
        try_files $uri $uri/ /index.html;
        add_header Cache-Control "no-cache";
    }
}
NGINX

COPY --from=build --chown=nginx:nginx /workspace/apps/studio/dist /usr/share/nginx/html

EXPOSE 8083
USER nginx

HEALTHCHECK --interval=15s --timeout=5s --start-period=5s --retries=5 \
    CMD ["/bin/sh", "-c", "wget -qO- http://127.0.0.1:8083/healthz >/dev/null 2>&1 || exit 1"]
