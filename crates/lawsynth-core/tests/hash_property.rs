use lawsynth_core::stable_hash;

#[test]
fn hashes_are_stable_and_distinguish_prefix_and_suffix_changes() {
    let original = stable_hash(b"trajectory:x:0,1,2");
    assert_eq!(original, stable_hash(b"trajectory:x:0,1,2"));
    assert_ne!(original, stable_hash(b"trajectory:x:0,1,3"));
    assert_ne!(original, stable_hash(b"x:0,1,2"));
}
