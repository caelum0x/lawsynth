# Compatibility

The renderer accepts decoded metadata with semantic format major `1`. Unknown
fields remain in `RenderedArtifact.data`; unsupported format majors fail before
rendering so a partial interpretation is not presented as correct.
