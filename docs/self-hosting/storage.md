# Storage

Artifacts are addressed by SHA-256 and stored under `objects/sha256/<first-two>/<next-two>/<digest>` below `Settings.object_root`. Uploads are base64 decoded, limited by `max_upload_bytes`, hashed, and committed with an atomic filesystem replacement. Re-uploading identical bytes returns the same digest and does not require duplicate storage.

Place the object root on a local filesystem controlled by the service account. Do not make it web-readable, because the content-addressed paths are not an authorization mechanism. Keep the directory out of source control and give it restrictive permissions appropriate to the host.

S3, GCS, signed URLs, lifecycle policy, distributed locks, and encryption-key management are not implemented. An external object-store integration must preserve the artifact hash and tenant authorization contract.
