from lawsynth_bench.runner import ingest
def test_ingest_creates_evidence_digest(rows):
    artifact = ingest(rows)
    assert len(artifact.digest) == 64 and artifact.observations == tuple(rows)
