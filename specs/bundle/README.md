# LawSynth bundle format 0.1

A `.lsworld` file is a deterministic, stored-entry ZIP archive carrying one validated continuous or discrete World IR object. The normative codec is `lawsynth-bundle`. It writes a fixed three-entry layout, validates archive structure and checksums when reading, then reconstructs a `World` or `DiscreteWorld` through their normal validators.

Format 0.1 is intentionally small: no compressed entries, ZIP64, comments, multi-disk archives, user attachments, signatures embedded in the archive, migrations, or version negotiation beyond the fixed manifest. A caller may use the standalone HMAC helper for bytes it already controls; that does not create a signed-bundle format.
