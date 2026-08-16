use lawsynth_artifact_service::{is_sha256_hex, sha256};

#[test]
fn sha256_is_a_stable_content_address() {
    let digest = sha256(b"artifact bytes");
    assert!(is_sha256_hex(&digest));
    assert_ne!(digest, sha256(b"artifact bytez"));
}
