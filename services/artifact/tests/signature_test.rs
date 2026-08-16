use lawsynth_artifact_service::BundleAuthenticator;

#[test]
fn bundle_authenticator_detects_modified_bundle_bytes() {
    let authenticator = BundleAuthenticator::new(b"local secret".to_vec());
    let signature = authenticator.authenticate(b"bundle");
    assert!(authenticator.verify(b"bundle", &signature).valid);
    assert!(!authenticator.verify(b"modified", &signature).valid);
}
