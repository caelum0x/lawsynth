# Export from a host application

Native simulation exports CSV through stdout, and `chart-core` can produce CSV or JSON chart data. A host application should create a manifest that ties any downloaded table or graphic to the bundle hash, scenario, engine version, units, and plot transformation.

Keep the raw trajectory export separate from a downsampled display series. Downsampling is appropriate for rendering but is not suitable as the authoritative scientific export. Validate filenames and do not let user-controlled labels escape an export directory.

The repository does not ship Studio PDF reports, cloud export, sharing links, signed artifacts, or a server-side export queue. Those are product features that require their own implementation and security review.
