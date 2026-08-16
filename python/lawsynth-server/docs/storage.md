# Storage

The local object store accepts bounded byte payloads and makes a content hash
the physical identity. It does not expose path-based uploads, preventing path
traversal and accidental tenant-path interpretation. S3 signed URLs need a
cloud adapter and are deliberately not fabricated by the local implementation.
