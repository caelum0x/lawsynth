# Air-gapped operation

The local server core can run without network access once its Python environment and package artifacts are installed. SQLite and filesystem storage require no cloud account. Prepare an offline wheelhouse or approved package mirror in advance, install dependencies from that source, and keep the object root and SQLite database on locally managed volumes.

Air-gapping does not remove operational controls. Provision bearer tokens through an offline secret-management process, restrict process and filesystem permissions, validate imported data before discovery, and export bundles or reports through a reviewed transfer channel.

There is no built-in offline update feed, package signing workflow, remote telemetry sink, or synchronization protocol. `telemetry_enabled` only governs the local core's telemetry behavior; it is not a managed observability service.
