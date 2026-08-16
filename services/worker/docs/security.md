# Security boundary

This crate trusts only its in-process caller. It does not deserialize untrusted queue
messages, bind a network port, authenticate principals, execute plugins, or claim an
OS sandbox. Resource declarations gate admission but cannot contain arbitrary code;
callers must establish isolation before placing untrusted workloads in a process.

The local store's checksum protects against accidental corruption, not a malicious
storage actor. Use an authenticated artifact layer when integrity against attackers
is a requirement.
