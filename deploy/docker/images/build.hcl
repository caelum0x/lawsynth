# Docker Buildx Bake definition for all LawSynth service images.
#
# Build the full set from the repository root:
#   docker buildx bake -f deploy/docker/images/build.hcl
#
# Build and push a released, multi-arch set:
#   REGISTRY=ghcr.io/lawsynth VERSION=0.1.0 \
#     docker buildx bake -f deploy/docker/images/build.hcl --set *.platform=linux/amd64,linux/arm64 release
#
# Build one image:
#   docker buildx bake -f deploy/docker/images/build.hcl api

variable "REGISTRY" {
  default = "ghcr.io/lawsynth"
}

variable "VERSION" {
  default = "0.1.0"
}

# Extra tag applied to every image (e.g. a git sha in CI). Empty by default.
variable "SHA_TAG" {
  default = ""
}

# The build context is the repository root, two levels up from this file.
variable "CONTEXT" {
  default = "../../.."
}

function "tags" {
  params = [name]
  result = SHA_TAG == "" ? [
    "${REGISTRY}/${name}:${VERSION}",
    "${REGISTRY}/${name}:latest",
  ] : [
    "${REGISTRY}/${name}:${VERSION}",
    "${REGISTRY}/${name}:latest",
    "${REGISTRY}/${name}:${SHA_TAG}",
  ]
}

# Reusable OCI provenance labels.
function "labels" {
  params = [name]
  result = {
    "org.opencontainers.image.title"    = "lawsynth-${name}"
    "org.opencontainers.image.version"  = "${VERSION}"
    "org.opencontainers.image.source"   = "https://github.com/lawsynth/lawsynth"
    "org.opencontainers.image.licenses" = "Apache-2.0"
    "org.opencontainers.image.vendor"   = "LawSynth"
  }
}

group "default" {
  targets = ["api", "gateway", "scheduler", "worker", "artifact", "studio"]
}

# Runtime service images only (excludes the heavyweight development image).
group "services" {
  targets = ["api", "gateway", "scheduler", "worker", "artifact", "studio"]
}

# CI/release group: same as services, driven by --set *.platform and --push.
group "release" {
  targets = ["api", "gateway", "scheduler", "worker", "artifact", "studio"]
}

target "_common" {
  context    = CONTEXT
  platforms  = ["linux/amd64"]
  pull       = true
}

target "api" {
  inherits   = ["_common"]
  dockerfile = "deploy/docker/images/api.Dockerfile"
  tags       = tags("api")
  labels     = labels("api")
}

target "gateway" {
  inherits   = ["_common"]
  dockerfile = "deploy/docker/images/gateway.Dockerfile"
  tags       = tags("gateway")
  labels     = labels("gateway")
}

target "scheduler" {
  inherits   = ["_common"]
  dockerfile = "deploy/docker/images/scheduler.Dockerfile"
  tags       = tags("scheduler")
  labels     = labels("scheduler")
}

target "worker" {
  inherits   = ["_common"]
  dockerfile = "deploy/docker/images/worker.Dockerfile"
  tags       = tags("worker")
  labels     = labels("worker")
}

target "artifact" {
  inherits   = ["_common"]
  dockerfile = "deploy/docker/images/artifact.Dockerfile"
  tags       = tags("artifact")
  labels     = labels("artifact")
}

target "studio" {
  inherits   = ["_common"]
  dockerfile = "deploy/docker/images/studio.Dockerfile"
  tags       = tags("studio")
  labels     = labels("studio")
}

target "development" {
  inherits   = ["_common"]
  dockerfile = "deploy/docker/images/development.Dockerfile"
  tags       = tags("development")
  labels     = labels("development")
}
