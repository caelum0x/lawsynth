# Artifacts

Artifacts are immutable bytes addressed by SHA-256. `FileObjectStore` writes
atomically beneath `objects/sha256/<first-two>/<next-two>/<hash>` and verifies
the key shape before reads. Artifact metadata records only hash, byte size, and
media type; database records should reference that hash rather than embed data.
