# Server boundary

`LocalArtifactProxy` serves only mappings supplied by the caller. The `connect`
function consistently raises `UnsupportedCapabilityError`: authenticated remote
API access belongs to a dedicated LawSynth client, not a notebook renderer.
