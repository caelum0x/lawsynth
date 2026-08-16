use std::fmt;

/// Byte offsets for the serialized Thrift metadata section in a Parquet file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParquetEnvelope {
    pub metadata_offset: usize,
    pub metadata_length: usize,
}

/// Errors detected before attempting Thrift metadata or column-page decoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParquetError {
    TooShort,
    MissingHeader,
    MissingFooter,
    InvalidMetadataLength,
}
impl fmt::Display for ParquetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort => write!(formatter, "Parquet file is too short"),
            Self::MissingHeader => write!(formatter, "Parquet header magic is missing"),
            Self::MissingFooter => write!(formatter, "Parquet footer magic is missing"),
            Self::InvalidMetadataLength => {
                write!(formatter, "Parquet metadata length exceeds the file")
            }
        }
    }
}
impl std::error::Error for ParquetError {}

/// Validates the standard `PAR1` header/footer envelope and locates metadata.
///
/// This deliberately stops before Thrift/page decoding. It is a safe, useful
/// preflight for callers that hand the metadata to a full Parquet codec, and
/// it never mistakes arbitrary bytes for a valid Parquet container.
pub fn inspect_parquet(bytes: &[u8]) -> Result<ParquetEnvelope, ParquetError> {
    if bytes.len() < 12 {
        return Err(ParquetError::TooShort);
    }
    if &bytes[..4] != b"PAR1" {
        return Err(ParquetError::MissingHeader);
    }
    if &bytes[bytes.len() - 4..] != b"PAR1" {
        return Err(ParquetError::MissingFooter);
    }
    let length_offset = bytes.len() - 8;
    let metadata_length = u32::from_le_bytes(
        bytes[length_offset..length_offset + 4]
            .try_into()
            .expect("four bytes"),
    ) as usize;
    let metadata_offset = length_offset
        .checked_sub(metadata_length)
        .ok_or(ParquetError::InvalidMetadataLength)?;
    if metadata_offset < 4 {
        return Err(ParquetError::InvalidMetadataLength);
    }
    Ok(ParquetEnvelope {
        metadata_offset,
        metadata_length,
    })
}

/// Decodes uncompressed, PLAIN-encoded required numeric Parquet columns.
///
/// The compact-Thrift metadata and data-page codec are implemented locally so
/// the initial build remains offline. Dictionary/RLE encodings, nullable and
/// repeated values, and compressed pages are rejected explicitly.
pub fn read_parquet_numeric(
    bytes: &[u8],
    time_column: &str,
) -> Result<crate::Dataset, crate::DataError> {
    let envelope =
        inspect_parquet(bytes).map_err(|error| crate::DataError::Parquet(error.to_string()))?;
    let metadata = parse_file_metadata(
        &bytes[envelope.metadata_offset..envelope.metadata_offset + envelope.metadata_length],
    )
    .map_err(crate::DataError::Parquet)?;
    let mut decoded = std::collections::BTreeMap::<String, Vec<f64>>::new();
    for group in metadata {
        for column in group {
            decoded
                .entry(column.name.clone())
                .or_default()
                .extend(decode_column(bytes, &column).map_err(crate::DataError::Parquet)?);
        }
    }
    let time = decoded
        .remove(time_column)
        .ok_or_else(|| crate::DataError::Parquet(format!("missing time column '{time_column}'")))?;
    let columns = decoded
        .into_iter()
        .map(|(name, values)| {
            lawsynth_core::Identifier::new(name)
                .map(|id| crate::NumericColumn::new(id, values))
                .map_err(|error| crate::DataError::Parquet(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    crate::Dataset::new(
        crate::TimeAxis::new(time).map_err(|error| crate::DataError::Parquet(error.to_string()))?,
        columns,
    )
}

#[derive(Clone)]
struct Column {
    name: String,
    physical: i32,
    codec: i32,
    offset: usize,
    size: usize,
    count: usize,
}
const STOP: u8 = 0;
const I32: u8 = 5;
const I64: u8 = 6;
const BINARY: u8 = 8;
const LIST: u8 = 9;
const STRUCT: u8 = 12;

fn parse_file_metadata(bytes: &[u8]) -> Result<Vec<Vec<Column>>, String> {
    let mut cursor = Compact::new(bytes);
    let mut groups = Vec::new();
    fields(&mut cursor, |id, ty, cursor| {
        if id == 4 && ty == LIST {
            let (count, item) = cursor.list()?;
            if item != STRUCT {
                return Err("row groups are not structs".into());
            }
            for _ in 0..count {
                groups.push(parse_group(cursor)?);
            }
            Ok(())
        } else {
            cursor.skip(ty)
        }
    })?;
    if groups.is_empty() {
        Err("metadata has no row groups".into())
    } else {
        Ok(groups)
    }
}
fn parse_group(cursor: &mut Compact<'_>) -> Result<Vec<Column>, String> {
    let mut result = Vec::new();
    fields(cursor, |id, ty, cursor| {
        if id == 1 && ty == LIST {
            let (count, item) = cursor.list()?;
            if item != STRUCT {
                return Err("column chunks are not structs".into());
            }
            for _ in 0..count {
                result.push(parse_chunk(cursor)?);
            }
            Ok(())
        } else {
            cursor.skip(ty)
        }
    })?;
    Ok(result)
}
fn parse_chunk(cursor: &mut Compact<'_>) -> Result<Column, String> {
    let mut column = None;
    fields(cursor, |id, ty, cursor| {
        if id == 3 && ty == STRUCT {
            column = Some(parse_column_metadata(cursor)?);
            Ok(())
        } else {
            cursor.skip(ty)
        }
    })?;
    column.ok_or_else(|| "column chunk lacks metadata".into())
}
fn parse_column_metadata(cursor: &mut Compact<'_>) -> Result<Column, String> {
    let (mut physical, mut name, mut codec, mut count, mut size, mut offset) =
        (None, None, None, None, None, None);
    fields(cursor, |id, ty, cursor| match id {
        1 if ty == I32 => {
            physical = Some(cursor.i32()?);
            Ok(())
        }
        3 if ty == LIST => {
            let (n, item) = cursor.list()?;
            if n != 1 || item != BINARY {
                return Err("only flat leaf columns are supported".into());
            }
            name = Some(cursor.string()?);
            Ok(())
        }
        4 if ty == I32 => {
            codec = Some(cursor.i32()?);
            Ok(())
        }
        5 if ty == I64 => {
            count = Some(cursor.i64()? as usize);
            Ok(())
        }
        7 if ty == I64 => {
            size = Some(cursor.i64()? as usize);
            Ok(())
        }
        9 if ty == I64 => {
            offset = Some(cursor.i64()? as usize);
            Ok(())
        }
        _ => cursor.skip(ty),
    })?;
    Ok(Column {
        name: name.ok_or("column path missing")?,
        physical: physical.ok_or("column type missing")?,
        codec: codec.ok_or("column codec missing")?,
        count: count.ok_or("column count missing")?,
        size: size.ok_or("column compressed size missing")?,
        offset: offset.ok_or("column data offset missing")?,
    })
}
fn decode_column(file: &[u8], column: &Column) -> Result<Vec<f64>, String> {
    if column.codec != 0 {
        return Err(format!(
            "column '{}' uses unsupported compression codec {}",
            column.name, column.codec
        ));
    }
    let end = column
        .offset
        .checked_add(column.size)
        .ok_or("column range overflows")?;
    let mut cursor = Compact::new(
        file.get(column.offset..end)
            .ok_or("column range exceeds file")?,
    );
    let mut output = Vec::with_capacity(column.count);
    while output.len() < column.count {
        let page = parse_page(&mut cursor)?;
        if page.kind != 0 || page.encoding != 0 {
            return Err("only PLAIN DATA_PAGE pages are supported".into());
        }
        if page.compressed != page.uncompressed {
            return Err("compressed page payloads are unsupported".into());
        }
        let body = cursor.take(page.compressed)?;
        let width = match column.physical {
            1 | 4 => 4,
            2 | 5 => 8,
            _ => return Err(format!("unsupported physical type {}", column.physical)),
        };
        if body.len()
            != page
                .values
                .checked_mul(width)
                .ok_or("page length overflows")?
        {
            return Err("page has definition/repetition levels or invalid PLAIN bytes".into());
        }
        for raw in body.chunks_exact(width) {
            output.push(match column.physical {
                1 => i32::from_le_bytes(raw.try_into().unwrap()) as f64,
                2 => i64::from_le_bytes(raw.try_into().unwrap()) as f64,
                4 => f32::from_le_bytes(raw.try_into().unwrap()) as f64,
                5 => f64::from_le_bytes(raw.try_into().unwrap()),
                _ => unreachable!(),
            });
        }
    }
    if output.len() != column.count {
        Err("page values do not match metadata count".into())
    } else {
        Ok(output)
    }
}
struct Page {
    kind: i32,
    uncompressed: usize,
    compressed: usize,
    values: usize,
    encoding: i32,
}
fn parse_page(cursor: &mut Compact<'_>) -> Result<Page, String> {
    let (mut kind, mut uncompressed, mut compressed, mut values, mut encoding) =
        (None, None, None, None, None);
    fields(cursor, |id, ty, cursor| match id {
        1 if ty == I32 => {
            kind = Some(cursor.i32()?);
            Ok(())
        }
        2 if ty == I32 => {
            uncompressed = Some(cursor.i32()? as usize);
            Ok(())
        }
        3 if ty == I32 => {
            compressed = Some(cursor.i32()? as usize);
            Ok(())
        }
        5 if ty == STRUCT => fields(cursor, |field, ty, cursor| match field {
            1 if ty == I32 => {
                values = Some(cursor.i32()? as usize);
                Ok(())
            }
            2 if ty == I32 => {
                encoding = Some(cursor.i32()?);
                Ok(())
            }
            _ => cursor.skip(ty),
        }),
        _ => cursor.skip(ty),
    })?;
    Ok(Page {
        kind: kind.ok_or("page type missing")?,
        uncompressed: uncompressed.ok_or("page size missing")?,
        compressed: compressed.ok_or("compressed page size missing")?,
        values: values.ok_or("page value count missing")?,
        encoding: encoding.ok_or("page encoding missing")?,
    })
}
fn fields(
    cursor: &mut Compact<'_>,
    mut visit: impl FnMut(i16, u8, &mut Compact<'_>) -> Result<(), String>,
) -> Result<(), String> {
    let mut prior = 0;
    loop {
        let header = cursor.byte()?;
        let ty = header & 15;
        if ty == STOP {
            return Ok(());
        }
        let delta = (header >> 4) as i16;
        let id = if delta == 0 {
            cursor.i16()?
        } else {
            prior + delta
        };
        prior = id;
        visit(id, ty, cursor)?;
    }
}
struct Compact<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Compact<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    fn take(&mut self, length: usize) -> Result<&'a [u8], String> {
        let end = self.position.checked_add(length).ok_or("length overflow")?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or("unexpected end of compact data")?;
        self.position = end;
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn varint(&mut self) -> Result<u64, String> {
        let mut value = 0;
        for shift in (0..64).step_by(7) {
            let byte = self.byte()?;
            value |= u64::from(byte & 127) << shift;
            if byte & 128 == 0 {
                return Ok(value);
            }
        }
        Err("varint overflow".into())
    }
    fn i32(&mut self) -> Result<i32, String> {
        let value = self.varint()?;
        Ok(((value >> 1) as i32) ^ -((value & 1) as i32))
    }
    fn i64(&mut self) -> Result<i64, String> {
        let value = self.varint()?;
        Ok(((value >> 1) as i64) ^ -((value & 1) as i64))
    }
    fn i16(&mut self) -> Result<i16, String> {
        Ok(self.i32()? as i16)
    }
    fn string(&mut self) -> Result<String, String> {
        let length = self.varint()? as usize;
        String::from_utf8(self.take(length)?.to_vec()).map_err(|_| "non-UTF8 column path".into())
    }
    fn list(&mut self) -> Result<(usize, u8), String> {
        let header = self.byte()?;
        let mut count = (header >> 4) as usize;
        let ty = header & 15;
        if count == 15 {
            count = self.varint()? as usize
        }
        Ok((count, ty))
    }
    fn skip(&mut self, ty: u8) -> Result<(), String> {
        match ty {
            1 | 2 => Ok(()),
            3 => {
                self.take(1)?;
                Ok(())
            }
            4..=6 => {
                self.varint()?;
                Ok(())
            }
            7 => {
                self.take(8)?;
                Ok(())
            }
            8 => {
                let length = self.varint()? as usize;
                self.take(length)?;
                Ok(())
            }
            9 | 10 => {
                let (count, item) = self.list()?;
                for _ in 0..count {
                    self.skip(item)?
                }
                Ok(())
            }
            11 => Err("compact maps are unsupported".into()),
            12 => fields(self, |_, ty, cursor| cursor.skip(ty)),
            _ => Err("unknown compact type".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn varint(mut value: u64, out: &mut Vec<u8>) {
        loop {
            let mut byte = (value & 127) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 128;
            }
            out.push(byte);
            if value == 0 {
                return;
            }
        }
    }

    fn i32(value: i32, out: &mut Vec<u8>) {
        varint(((value << 1) ^ (value >> 31)) as u32 as u64, out);
    }

    fn i64(value: i64, out: &mut Vec<u8>) {
        varint(((value << 1) ^ (value >> 63)) as u64, out);
    }

    fn plain_double_page(values: &[f64]) -> Vec<u8> {
        let body = values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<_>>();
        let length = body.len() as i32;
        let mut page = vec![0x15]; // PageHeader.type = DATA_PAGE
        i32(0, &mut page);
        page.push(0x15); // uncompressed_page_size
        i32(length, &mut page);
        page.push(0x15); // compressed_page_size
        i32(length, &mut page);
        page.push(0x2c); // DataPageHeader (field 5)
        page.push(0x15); // num_values
        i32(values.len() as i32, &mut page);
        page.push(0x15); // encoding = PLAIN
        i32(0, &mut page);
        page.push(0);
        page.push(0);
        page.extend(body);
        page
    }

    fn column_chunk(name: &str, page_offset: usize, page_length: usize) -> Vec<u8> {
        let mut chunk = vec![0x3c]; // ColumnChunk.meta_data (field 3)
        chunk.push(0x15); // ColumnMetaData.type = DOUBLE
        i32(5, &mut chunk);
        chunk.push(0x29); // path_in_schema (field 3)
        chunk.push(0x18); // one BINARY item
        varint(name.len() as u64, &mut chunk);
        chunk.extend(name.as_bytes());
        chunk.push(0x15); // codec = UNCOMPRESSED (field 4)
        i32(0, &mut chunk);
        chunk.push(0x16); // num_values (field 5)
        i64(2, &mut chunk);
        chunk.push(0x26); // total_compressed_size (field 7)
        i64(page_length as i64, &mut chunk);
        chunk.push(0x26); // data_page_offset (field 9)
        i64(page_offset as i64, &mut chunk);
        chunk.push(0);
        chunk.push(0);
        chunk
    }

    fn minimal_plain_double_file() -> Vec<u8> {
        let time_page = plain_double_page(&[0.0, 1.0]);
        let value_page = plain_double_page(&[2.0, 4.0]);
        let time_offset = 4;
        let value_offset = time_offset + time_page.len();
        let mut metadata = vec![0x49, 0x1c]; // FileMetaData.row_groups: one struct
        metadata.push(0x19); // RowGroup.columns (field 1)
        metadata.push(0x2c); // two STRUCT items
        metadata.extend(column_chunk("time", time_offset, time_page.len()));
        metadata.extend(column_chunk("x", value_offset, value_page.len()));
        metadata.push(0);
        metadata.push(0);

        let mut file = b"PAR1".to_vec();
        file.extend(time_page);
        file.extend(value_page);
        file.extend(&metadata);
        file.extend((metadata.len() as u32).to_le_bytes());
        file.extend(b"PAR1");
        file
    }
    #[test]
    fn locates_valid_envelope_metadata() {
        let mut bytes = b"PAR1meta".to_vec();
        bytes.extend((4u32).to_le_bytes());
        bytes.extend(b"PAR1");
        assert_eq!(
            inspect_parquet(&bytes).unwrap(),
            ParquetEnvelope {
                metadata_offset: 4,
                metadata_length: 4
            }
        );
    }

    #[test]
    fn decodes_uncompressed_plain_double_columns() {
        let dataset = read_parquet_numeric(&minimal_plain_double_file(), "time").unwrap();
        assert_eq!(dataset.time().values(), &[0.0, 1.0]);
        assert_eq!(
            dataset.columns()[&lawsynth_core::Identifier::new("x").unwrap()].values,
            &[2.0, 4.0]
        );
    }
}
