use std::collections::BTreeMap;

use crate::BundleError;

const LOCAL_HEADER: u32 = 0x0403_4b50;
const CENTRAL_HEADER: u32 = 0x0201_4b50;
const END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;

pub(crate) fn write_archive(entries: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, BundleError> {
    let mut output = Vec::new();
    let mut central_directory = Vec::new();

    for (path, content) in entries {
        validate_path(path)?;
        let name = path.as_bytes();
        let name_length = to_u16(name.len(), "entry path is too long")?;
        let size = to_u32(content.len(), "entry is too large")?;
        let offset = to_u32(output.len(), "archive is too large")?;
        let checksum = crc32(content);

        put_u32(&mut output, LOCAL_HEADER);
        put_u16(&mut output, 20);
        put_u16(&mut output, 0);
        put_u16(&mut output, 0);
        put_u16(&mut output, 0);
        put_u16(&mut output, 0);
        put_u32(&mut output, checksum);
        put_u32(&mut output, size);
        put_u32(&mut output, size);
        put_u16(&mut output, name_length);
        put_u16(&mut output, 0);
        output.extend(name);
        output.extend(content);

        put_u32(&mut central_directory, CENTRAL_HEADER);
        put_u16(&mut central_directory, 20);
        put_u16(&mut central_directory, 20);
        put_u16(&mut central_directory, 0);
        put_u16(&mut central_directory, 0);
        put_u16(&mut central_directory, 0);
        put_u16(&mut central_directory, 0);
        put_u32(&mut central_directory, checksum);
        put_u32(&mut central_directory, size);
        put_u32(&mut central_directory, size);
        put_u16(&mut central_directory, name_length);
        put_u16(&mut central_directory, 0);
        put_u16(&mut central_directory, 0);
        put_u16(&mut central_directory, 0);
        put_u16(&mut central_directory, 0);
        put_u32(&mut central_directory, 0);
        put_u32(&mut central_directory, offset);
        central_directory.extend(name);
    }

    let central_offset = to_u32(output.len(), "archive is too large")?;
    let central_size = to_u32(central_directory.len(), "archive is too large")?;
    let entry_count = to_u16(entries.len(), "too many archive entries")?;
    output.extend(central_directory);
    put_u32(&mut output, END_OF_CENTRAL_DIRECTORY);
    put_u16(&mut output, 0);
    put_u16(&mut output, 0);
    put_u16(&mut output, entry_count);
    put_u16(&mut output, entry_count);
    put_u32(&mut output, central_size);
    put_u32(&mut output, central_offset);
    put_u16(&mut output, 0);
    Ok(output)
}

pub(crate) fn read_archive(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, BundleError> {
    if bytes.len() < 22 {
        return Err(BundleError::InvalidArchive("missing end of central directory"));
    }
    let end_offset = bytes
        .len()
        .checked_sub(22)
        .ok_or(BundleError::InvalidArchive("missing end of central directory"))?;
    if read_u32(bytes, end_offset)? != END_OF_CENTRAL_DIRECTORY {
        return Err(BundleError::InvalidArchive(
            "archive comments and ZIP64 are not supported by this format version",
        ));
    }
    if read_u16(bytes, end_offset + 4)? != 0
        || read_u16(bytes, end_offset + 6)? != 0
        || read_u16(bytes, end_offset + 20)? != 0
    {
        return Err(BundleError::InvalidArchive("multi-disk archives are not supported"));
    }
    let count = usize::from(read_u16(bytes, end_offset + 10)?);
    let central_size = usize::try_from(read_u32(bytes, end_offset + 12)?)
        .map_err(|_| BundleError::InvalidArchive("central directory is too large"))?;
    let central_offset = usize::try_from(read_u32(bytes, end_offset + 16)?)
        .map_err(|_| BundleError::InvalidArchive("central directory offset is invalid"))?;
    if central_offset.checked_add(central_size).filter(|end| *end == end_offset).is_none() {
        return Err(BundleError::InvalidArchive("central directory bounds are invalid"));
    }

    let mut offset = central_offset;
    let mut entries = BTreeMap::new();
    for _ in 0..count {
        if read_u32(bytes, offset)? != CENTRAL_HEADER {
            return Err(BundleError::InvalidArchive("invalid central directory header"));
        }
        if read_u16(bytes, offset + 10)? != 0 {
            return Err(BundleError::InvalidArchive("compressed entries are not supported"));
        }
        let checksum = read_u32(bytes, offset + 16)?;
        let compressed_size = usize::try_from(read_u32(bytes, offset + 20)?)
            .map_err(|_| BundleError::InvalidArchive("entry is too large"))?;
        let uncompressed_size = usize::try_from(read_u32(bytes, offset + 24)?)
            .map_err(|_| BundleError::InvalidArchive("entry is too large"))?;
        if compressed_size != uncompressed_size {
            return Err(BundleError::InvalidArchive("stored entry has inconsistent sizes"));
        }
        let name_length = usize::from(read_u16(bytes, offset + 28)?);
        let extra_length = usize::from(read_u16(bytes, offset + 30)?);
        let comment_length = usize::from(read_u16(bytes, offset + 32)?);
        let local_offset = usize::try_from(read_u32(bytes, offset + 42)?)
            .map_err(|_| BundleError::InvalidArchive("local entry offset is invalid"))?;
        let name_start = offset + 46;
        let name_end = name_start
            .checked_add(name_length)
            .ok_or(BundleError::InvalidArchive("entry path is too long"))?;
        let path = std::str::from_utf8(
            bytes
                .get(name_start..name_end)
                .ok_or(BundleError::InvalidArchive("entry path exceeds archive"))?,
        )
        .map_err(|_| BundleError::InvalidArchive("entry path is not UTF-8"))?
        .to_owned();
        validate_path(&path)?;
        if read_u32(bytes, local_offset)? != LOCAL_HEADER {
            return Err(BundleError::InvalidArchive("invalid local entry header"));
        }
        if read_u16(bytes, local_offset + 8)? != 0 {
            return Err(BundleError::InvalidArchive("compressed entries are not supported"));
        }
        let local_name_length = usize::from(read_u16(bytes, local_offset + 26)?);
        let local_extra_length = usize::from(read_u16(bytes, local_offset + 28)?);
        let data_start = local_offset
            .checked_add(30 + local_name_length + local_extra_length)
            .ok_or(BundleError::InvalidArchive("entry data offset is invalid"))?;
        let data_end = data_start
            .checked_add(uncompressed_size)
            .ok_or(BundleError::InvalidArchive("entry is too large"))?;
        let content = bytes
            .get(data_start..data_end)
            .ok_or(BundleError::InvalidArchive("entry data exceeds archive"))?
            .to_vec();
        if crc32(&content) != checksum {
            return Err(BundleError::ChecksumMismatch(path));
        }
        if entries.insert(path.clone(), content).is_some() {
            return Err(BundleError::InvalidPath(path));
        }
        offset = name_end
            .checked_add(extra_length + comment_length)
            .ok_or(BundleError::InvalidArchive("central directory entry is too large"))?;
    }
    if offset != central_offset + central_size {
        return Err(BundleError::InvalidArchive("central directory has trailing data"));
    }
    Ok(entries)
}

fn validate_path(path: &str) -> Result<(), BundleError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.as_bytes().contains(&92)
        || path.split('/').any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(BundleError::InvalidPath(path.to_owned()));
    }
    Ok(())
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

fn to_u16(value: usize, reason: &'static str) -> Result<u16, BundleError> {
    u16::try_from(value).map_err(|_| BundleError::InvalidArchive(reason))
}

fn to_u32(value: usize, reason: &'static str) -> Result<u32, BundleError> {
    u32::try_from(value).map_err(|_| BundleError::InvalidArchive(reason))
}

fn put_u16(output: &mut Vec<u8>, value: u16) {
    output.extend(value.to_le_bytes());
}

fn put_u32(output: &mut Vec<u8>, value: u32) {
    output.extend(value.to_le_bytes());
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, BundleError> {
    let slice = bytes
        .get(offset..offset + 2)
        .ok_or(BundleError::InvalidArchive("unexpected end of archive"))?;
    Ok(u16::from_le_bytes(slice.try_into().unwrap()))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, BundleError> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or(BundleError::InvalidArchive("unexpected end of archive"))?;
    Ok(u32::from_le_bytes(slice.try_into().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::{read_archive, write_archive};
    use std::collections::BTreeMap;

    #[test]
    fn archive_round_trips_in_sorted_order() {
        let entries = BTreeMap::from([
            ("world/world.bin".to_owned(), vec![3, 1, 4]),
            ("manifest.json".to_owned(), b"{}".to_vec()),
        ]);
        assert_eq!(read_archive(&write_archive(&entries).unwrap()).unwrap(), entries);
    }
}
