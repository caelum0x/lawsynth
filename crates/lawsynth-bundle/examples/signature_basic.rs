use lawsynth_bundle::{BundleSignature, verify_signature};

fn main() {
    let world_bytes = b"deterministic bundle payload";
    let tag = BundleSignature::authenticate(b"demo-shared-secret", world_bytes);
    assert!(verify_signature(b"demo-shared-secret", world_bytes, &tag));
    assert!(!verify_signature(b"demo-shared-secret", b"changed payload", &tag));
    println!("authenticated bundle with HMAC-SHA256 tag {}", tag.0);
}
