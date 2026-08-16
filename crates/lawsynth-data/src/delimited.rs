//! RFC-4180-style delimited numeric ingestion for CSV and TSV observations.

use crate::{DataError, Dataset, NumericColumn, TimeAxis};
use lawsynth_core::Identifier;

/// Decodes a UTF-8 CSV file with a header and a named time column.
pub fn read_csv_numeric(bytes: &[u8], time_column: &str) -> Result<Dataset, DataError> {
    read_delimited_numeric(bytes, b',', time_column)
}

/// Decodes a UTF-8 TSV file with a header and a named time column.
pub fn read_tsv_numeric(bytes: &[u8], time_column: &str) -> Result<Dataset, DataError> {
    read_delimited_numeric(bytes, b'\t', time_column)
}

/// Decodes quoted, escaped, rectangular numeric delimited text.
///
/// Delimiters and newlines inside double-quoted fields are preserved; a quote
/// inside a quoted field must be escaped as `""`. Fields are trimmed before
/// identifier and numeric validation so conventional human-authored headers
/// such as `time, x` retain their expected meaning.
pub fn read_delimited_numeric(
    bytes: &[u8],
    delimiter: u8,
    time_column: &str,
) -> Result<Dataset, DataError> {
    if delimiter == b'\n' || delimiter == b'\r' || delimiter == b'"' {
        return Err(delimited("delimiter must not be a quote or newline"));
    }
    let records = parse_records(bytes, delimiter)?;
    let (header, rows) = records.split_first().ok_or_else(|| delimited("input has no header"))?;
    if header.is_empty() || header.iter().any(|field| field.trim().is_empty()) {
        return Err(delimited("header has an empty column name"));
    }
    let header = header
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let field = field.trim();
            let field =
                if index == 0 { field.strip_prefix('\u{feff}').unwrap_or(field) } else { field };
            Identifier::new(field)
                .map_err(|error| delimited(format!("invalid header '{field}': {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let time_index = header
        .iter()
        .position(|name| name.as_str() == time_column)
        .ok_or_else(|| delimited(format!("missing time column '{time_column}'")))?;
    let mut values = vec![Vec::with_capacity(rows.len()); header.len()];
    for (record_index, row) in rows.iter().enumerate() {
        if row.len() != header.len() {
            return Err(delimited(format!(
                "record {} has {} fields; expected {}",
                record_index + 2,
                row.len(),
                header.len()
            )));
        }
        for (column_index, field) in row.iter().enumerate() {
            let value = field.trim().parse::<f64>().map_err(|_| {
                delimited(format!(
                    "record {}, column '{}' is not a number",
                    record_index + 2,
                    header[column_index]
                ))
            })?;
            values[column_index].push(value);
        }
    }
    let time = TimeAxis::new(values[time_index].clone())?;
    let columns = header
        .into_iter()
        .zip(values)
        .enumerate()
        .filter(|(index, _)| *index != time_index)
        .map(|(_, (id, values))| NumericColumn::new(id, values))
        .collect::<Vec<_>>();
    Dataset::new(time, columns)
}

fn delimited(reason: impl Into<String>) -> DataError {
    DataError::Delimited(reason.into())
}

fn parse_records(bytes: &[u8], delimiter: u8) -> Result<Vec<Vec<String>>, DataError> {
    let text = std::str::from_utf8(bytes).map_err(|_| delimited("input is not valid UTF-8"))?;
    let mut records = Vec::new();
    let mut record = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut after_quote = false;
    let mut saw_content = false;
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if quoted {
            if character == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                    after_quote = true;
                }
            } else {
                field.push(character);
            }
            continue;
        }
        if after_quote {
            match character {
                c if c as u32 == delimiter as u32 => {
                    record.push(std::mem::take(&mut field));
                    after_quote = false;
                    saw_content = true;
                }
                '\n' => {
                    record.push(std::mem::take(&mut field));
                    records.push(std::mem::take(&mut record));
                    after_quote = false;
                    saw_content = false;
                }
                '\r' if chars.peek() == Some(&'\n') => {
                    chars.next();
                    record.push(std::mem::take(&mut field));
                    records.push(std::mem::take(&mut record));
                    after_quote = false;
                    saw_content = false;
                }
                _ => {
                    return Err(delimited(
                        "characters after a closing quote must be a delimiter or newline",
                    ));
                }
            }
            continue;
        }
        match character {
            '"' if field.is_empty() => {
                quoted = true;
                saw_content = true;
            }
            '"' => return Err(delimited("quote inside an unquoted field")),
            c if c as u32 == delimiter as u32 => {
                record.push(std::mem::take(&mut field));
                saw_content = true;
            }
            '\n' => {
                record.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut record));
                saw_content = false;
            }
            '\r' if chars.peek() == Some(&'\n') => {
                chars.next();
                record.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut record));
                saw_content = false;
            }
            '\r' => return Err(delimited("bare carriage return is not a record separator")),
            character => {
                field.push(character);
                saw_content = true;
            }
        }
    }
    if quoted {
        return Err(delimited("unterminated quoted field"));
    }
    if saw_content || !field.is_empty() || !record.is_empty() {
        record.push(field);
        records.push(record);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::{read_csv_numeric, read_tsv_numeric};
    use lawsynth_core::Identifier;

    #[test]
    fn decodes_quoted_csv_records_and_a_utf8_bom() {
        let dataset = read_csv_numeric(b"\xef\xbb\xbftime,x\r\n0,1\r\n1,2\r\n", "time").unwrap();
        assert_eq!(dataset.time().values(), &[0.0, 1.0]);
        assert_eq!(dataset.columns()[&Identifier::new("x").unwrap()].values, &[1.0, 2.0]);
    }

    #[test]
    fn decodes_tsv_and_rejects_non_rectangular_records() {
        let dataset = read_tsv_numeric(b"time\tx\n0\t1\n1\t2\n", "time").unwrap();
        assert_eq!(dataset.columns()[&Identifier::new("x").unwrap()].values, &[1.0, 2.0]);
        let error = read_csv_numeric(b"time,x\n0,1,2\n", "time").unwrap_err();
        assert!(error.to_string().contains("record 2 has 3 fields"));
    }

    #[test]
    fn rejects_malformed_quotes_instead_of_silently_retokenizing() {
        let error = read_csv_numeric(b"time,x\n0,\"1\"suffix\n", "time").unwrap_err();
        assert!(error.to_string().contains("characters after a closing quote"));
    }
}
