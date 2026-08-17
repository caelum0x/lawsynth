# Plugin marketplace boundary (P8)

This directory specifies safe distribution and installation of community
extensions (connectors, feature libraries, operators, exporters, pipeline
stages). It is a **boundary specification** extending the implemented
`lawsynth-plugin-api`/`lawsynth-plugin-host` and `specs/plugin-protocol/`.

## Package format

A distributable plugin MUST be a self-describing package containing: the plugin
manifest (the existing `plugin.manifest` grammar), the plugin artifact(s)
(WASI module, process binary reference, or Python package), a declared version
(semver), a declared capability set, and a checksum manifest (SHA-256 per file).
The package hash is the content hash of its checksum manifest — deterministic and
mirror-stable.

## Signing & trust

A package MAY be signed; the signature covers the package hash. An installer MUST
verify a signature against a configured trust set before granting any capability
beyond `none`, and MUST record the verified signer with the installed plugin. An
unsigned or unverifiable package MUST be installable only with an explicit
`--allow-unverified` acknowledgement and MUST be marked untrusted.

## Capability grants

Declaration is not permission (per `specs/plugin-protocol/permissions.md`). At
install time the user grants a subset of the manifest's declared capabilities;
the host MUST enforce the granted subset at runtime and deny undeclared or
ungranted capabilities. Resource limits (`max_cpu_millis`, `max_memory_bytes`,
`max_output_bytes`, `max_requests`) MUST be enforced by the host, not trusted
from the plugin.

## Registry & mirroring

A registry is an **index** (a directory/file mapping plugin id + version → package
hash + location + signer), not a mandatory hosted service. A conforming registry
MUST be mirrorable offline: given the index and the packages, an air-gapped host
can install and verify without network access. The reference registry format MUST
be plain, diffable text with integrity hashes.

## Local commands

An implementation SHOULD expose `plugin install <pkg>`, `plugin list`,
`plugin verify <id>`, `plugin remove <id>`, all deterministic and offline against
a local index. Install MUST be non-destructive (refuse to replace a different
version without `--force`) and MUST verify checksums before activation.

## Honesty

The host's plugin/sandbox seams are documented as seams where OS-level isolation
is not yet linked. A marketplace implementation MUST NOT claim isolation it does
not provide; it MUST state exactly which enforcement is real (capability grants,
resource limits, checksum/signature verification) versus advisory.
