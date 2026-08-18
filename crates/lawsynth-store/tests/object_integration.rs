use lawsynth_store::{MemoryStore, MultipartUpload, ObjectKey, ObjectStore, collect_unreferenced};
use std::collections::BTreeSet;
#[test]
fn multipart_commit_and_gc_preserve_reachable_data() {
    let store = MemoryStore::default();
    let retained = ObjectKey::new("bundle/one").unwrap();
    let mut upload = MultipartUpload::new(retained.clone(), 4).unwrap();
    upload.add_part(1, b"law".to_vec()).unwrap();
    upload.add_part(2, b"synth".to_vec()).unwrap_err();
    upload.add_part(2, b"data".to_vec()).unwrap();
    assert_eq!(upload.complete(&store).unwrap().bytes, b"lawdata");
    store.put(ObjectKey::new("bundle/orphan").unwrap(), b"old".to_vec()).unwrap();
    let report =
        collect_unreferenced(&store, &BTreeSet::from([retained.clone()]), Some("bundle/"), false)
            .unwrap();
    assert_eq!(report.deleted, vec![ObjectKey::new("bundle/orphan").unwrap()]);
    assert_eq!(store.get(&retained).unwrap().bytes, b"lawdata");
}
