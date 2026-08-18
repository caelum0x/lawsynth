use std::fs;

use lawsynth_bundle::{BundleError, read_world, write_world};
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
        [ContinuousLaw::new(id("x"), Expr::symbol(id("x")))],
    )
    .unwrap()
}

fn temporary_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "lawsynth-bundle-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ))
}

fn read_u16(bytes: &[u8], offset: usize) -> usize {
    usize::from(u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap()))
}

fn read_u32(bytes: &[u8], offset: usize) -> usize {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut checksum = !0_u32;
    for byte in bytes {
        checksum ^= u32::from(*byte);
        for _ in 0..8 {
            checksum =
                if checksum & 1 == 1 { (checksum >> 1) ^ 0xedb8_8320 } else { checksum >> 1 };
        }
    }
    !checksum
}

/// Replaces equal-length entry content while maintaining the ZIP's local and central CRCs.
/// This lets the reader reach manifest validation instead of failing archive integrity first.
fn replace_stored_entry(bytes: &mut [u8], entry: &str, replacement: &[u8]) {
    let end = bytes.len() - 22;
    let mut central = read_u32(bytes, end + 16);
    let count = read_u16(bytes, end + 10);
    for _ in 0..count {
        assert_eq!(&bytes[central..central + 4], b"PK\x01\x02");
        let name_len = read_u16(bytes, central + 28);
        let extra_len = read_u16(bytes, central + 30);
        let comment_len = read_u16(bytes, central + 32);
        let name_start = central + 46;
        let name_end = name_start + name_len;
        if &bytes[name_start..name_end] == entry.as_bytes() {
            let local = read_u32(bytes, central + 42);
            let local_name_len = read_u16(bytes, local + 26);
            let local_extra_len = read_u16(bytes, local + 28);
            let data_start = local + 30 + local_name_len + local_extra_len;
            let size = read_u32(bytes, central + 24);
            assert_eq!(replacement.len(), size);
            bytes[data_start..data_start + size].copy_from_slice(replacement);
            let checksum = crc32(replacement).to_le_bytes();
            bytes[local + 14..local + 18].copy_from_slice(&checksum);
            bytes[central + 16..central + 20].copy_from_slice(&checksum);
            return;
        }
        central = name_end + extra_len + comment_len;
    }
    panic!("missing archive entry {entry}");
}

#[test]
fn written_bundle_contains_the_versioned_manifest_and_rejects_a_different_manifest() {
    let path = temporary_path("manifest");
    write_world(&path, &world()).unwrap();
    let mut archive = fs::read(&path).unwrap();

    let expected = b"{\n  \"format\": \"lawsynth-world\",\n  \"format_version\": \"0.1.0\",\n  \"world_encoding\": \"lawsynth-world-binary-v1\"\n}\n";
    replace_stored_entry(&mut archive, "manifest.json", expected);
    assert_eq!(read_world(&path).unwrap(), world());

    let unsupported = b"{\n  \"format\": \"lawsynth-world\",\n  \"format_version\": \"0.1.1\",\n  \"world_encoding\": \"lawsynth-world-binary-v1\"\n}\n";
    replace_stored_entry(&mut archive, "manifest.json", unsupported);
    fs::write(&path, archive).unwrap();
    assert!(matches!(read_world(&path), Err(BundleError::InvalidArchive("unsupported manifest"))));
    fs::remove_file(path).unwrap();
}
