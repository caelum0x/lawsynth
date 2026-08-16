use lawsynth_artifact_service::{ArtifactId, is_sha256_hex, sha256};

#[test]
fn artifact_ids_are_canonical_sha256_addresses() {
    let digest = sha256(b"world bundle");
    assert!(is_sha256_hex(&digest));
    assert_eq!(ArtifactId::new(digest.clone()).unwrap().as_str(), digest);
    assert!(ArtifactId::new("f".repeat(63)).is_err());
}
