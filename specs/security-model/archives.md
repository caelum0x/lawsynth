# Archive input handling

Format-0.1 `.lsworld` accepts only a single-disk, comment-free, stored-entry
ZIP layout. The reader rejects compressed entries, ZIP64, multi-disk archives,
invalid offsets, non-UTF-8 or unsafe paths, duplicate paths, inconsistent
sizes, CRC mismatches, unsupported manifests, malformed checksums, and invalid
World encodings. Binary expression nesting is capped below 128 and binary
string/count fields use checked parsing.

`BundleConfig` exposes intended aggregate limits (64 entries, 64 MiB per entry,
256 MiB total) but `read_world` and `read_discrete_world` do not accept it and
therefore do not enforce those aggregate values. A service receiving untrusted
files must impose file-size and resource limits before calling the reader.
