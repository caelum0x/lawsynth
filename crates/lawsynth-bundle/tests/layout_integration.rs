use std::fs;

use lawsynth_bundle::{read_world, write_world};
use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn world() -> World {
    World::new(
        [Variable::new(id("x"), VariableRole::State)],
        [],
        [ContinuousLaw::new(id("x"), Expr::constant(-0.25))],
    )
    .unwrap()
}

fn temporary_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "lawsynth-layout-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ))
}

fn u16_at(bytes: &[u8], offset: usize) -> usize {
    usize::from(u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap()))
}

fn u32_at(bytes: &[u8], offset: usize) -> usize {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize
}

#[test]
fn writer_emits_a_deterministic_stored_zip_with_the_canonical_bundle_layout() {
    let first = temporary_path("first");
    let second = temporary_path("second");
    let expected = world();
    write_world(&first, &expected).unwrap();
    write_world(&second, &expected).unwrap();
    let bytes = fs::read(&first).unwrap();

    assert_eq!(bytes, fs::read(&second).unwrap());
    assert_eq!(&bytes[..4], b"PK\x03\x04");
    let eocd = bytes.len() - 22;
    assert_eq!(&bytes[eocd..eocd + 4], b"PK\x05\x06");
    assert_eq!(u16_at(&bytes, eocd + 10), 3);

    let mut central = u32_at(&bytes, eocd + 16);
    let mut entries = Vec::new();
    for _ in 0..u16_at(&bytes, eocd + 10) {
        assert_eq!(&bytes[central..central + 4], b"PK\x01\x02");
        assert_eq!(u16_at(&bytes, central + 10), 0, "entries must be stored");
        let name_len = u16_at(&bytes, central + 28);
        let extra_len = u16_at(&bytes, central + 30);
        let comment_len = u16_at(&bytes, central + 32);
        let name_start = central + 46;
        let name_end = name_start + name_len;
        entries.push(std::str::from_utf8(&bytes[name_start..name_end]).unwrap());
        central = name_end + extra_len + comment_len;
    }
    assert_eq!(entries, ["manifest.json", "provenance/checksums.sha256", "world/world.bin"]);
    assert_eq!(read_world(&first).unwrap(), expected);
    fs::remove_file(first).unwrap();
    fs::remove_file(second).unwrap();
}
