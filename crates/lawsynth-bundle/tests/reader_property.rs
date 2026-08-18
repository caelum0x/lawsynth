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
        [ContinuousLaw::new(id("x"), Expr::difference(Expr::constant(2.0), Expr::symbol(id("x"))))],
    )
    .unwrap()
}

fn temporary_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "lawsynth-reader-{name}-{}-{}",
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

fn stored_entry_range(bytes: &[u8], wanted: &str) -> std::ops::Range<usize> {
    let eocd = bytes.len() - 22;
    let mut central = u32_at(bytes, eocd + 16);
    for _ in 0..u16_at(bytes, eocd + 10) {
        let name_len = u16_at(bytes, central + 28);
        let extra_len = u16_at(bytes, central + 30);
        let comment_len = u16_at(bytes, central + 32);
        let name_start = central + 46;
        let name_end = name_start + name_len;
        if &bytes[name_start..name_end] == wanted.as_bytes() {
            let local = u32_at(bytes, central + 42);
            let local_name_len = u16_at(bytes, local + 26);
            let local_extra_len = u16_at(bytes, local + 28);
            let start = local + 30 + local_name_len + local_extra_len;
            return start..start + u32_at(bytes, central + 24);
        }
        central = name_end + extra_len + comment_len;
    }
    panic!("missing archive entry {wanted}");
}

#[test]
fn reader_rejects_every_sampled_single_byte_tamper_of_the_world_payload() {
    let path = temporary_path("property");
    let expected = world();
    write_world(&path, &expected).unwrap();
    let pristine = fs::read(&path).unwrap();
    let world_range = stored_entry_range(&pristine, "world/world.bin");
    assert!(world_range.len() >= 8);

    for relative in [0, 1, world_range.len() / 2, world_range.len() - 1] {
        let mut tampered = pristine.clone();
        tampered[world_range.start + relative] ^= 0x80;
        fs::write(&path, tampered).unwrap();
        assert!(matches!(
            read_world(&path),
            Err(BundleError::ChecksumMismatch(ref entry)) if entry == "world/world.bin"
        ));
    }

    fs::write(&path, pristine).unwrap();
    assert_eq!(read_world(&path).unwrap(), expected);
    fs::remove_file(path).unwrap();
}
