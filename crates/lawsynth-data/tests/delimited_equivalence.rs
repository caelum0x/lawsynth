//! Proves the optimized single-pass delimited loader produces byte-for-byte
//! identical datasets and fingerprints to the original reference parser across
//! a wide range of well-formed and malformed inputs.
//!
//! The reference below is the pre-optimization implementation kept verbatim so
//! the equivalence assertion is meaningful: any divergence (values, ordering,
//! fingerprint, or error) fails the test.

use lawsynth_core::Identifier;
use lawsynth_data::{DataError, Dataset, NumericColumn, TimeAxis, read_csv_numeric};

// ---------------------------------------------------------------------------
// Reference implementation (original two-pass parser), kept for equivalence.
// ---------------------------------------------------------------------------

fn reference_read_csv(bytes: &[u8], time_column: &str) -> Result<Dataset, DataError> {
    reference_read_delimited(bytes, b',', time_column)
}

fn reference_read_delimited(
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

// ---------------------------------------------------------------------------
// Equivalence assertions.
// ---------------------------------------------------------------------------

fn assert_equivalent(bytes: &[u8], time_column: &str) {
    let reference = reference_read_csv(bytes, time_column);
    let optimized = read_csv_numeric(bytes, time_column);
    match (reference, optimized) {
        (Ok(reference), Ok(optimized)) => {
            assert_eq!(
                reference.fingerprint(),
                optimized.fingerprint(),
                "fingerprint mismatch for input {:?}",
                String::from_utf8_lossy(bytes)
            );
            assert_eq!(
                reference,
                optimized,
                "dataset mismatch for input {:?}",
                String::from_utf8_lossy(bytes)
            );
        }
        (Err(reference), Err(optimized)) => {
            assert_eq!(
                reference.to_string(),
                optimized.to_string(),
                "error mismatch for input {:?}",
                String::from_utf8_lossy(bytes)
            );
        }
        (reference, optimized) => panic!(
            "outcome mismatch for input {:?}: reference={:?} optimized={:?}",
            String::from_utf8_lossy(bytes),
            reference.map(|dataset| dataset.fingerprint()),
            optimized.map(|dataset| dataset.fingerprint()),
        ),
    }
}

#[test]
fn matches_reference_on_curated_edge_cases() {
    let cases: &[&[u8]] = &[
        b"time,x\n0,1\n1,2\n",
        b"time,x\n0,1\n1,2",                // no trailing newline
        b"time,x\r\n0,1\r\n1,2\r\n",        // CRLF
        b"\xef\xbb\xbftime,x\n0,1\n1,2\n",  // BOM
        b"time,x,y\n0,1,2\n1,3,4\n2,5,6\n", // multiple columns
        b"time,\"x\"\n0,1\n1,2\n",          // quoted header
        b"time,x\n0,\"1\"\n1,\"2\"\n",      // quoted values
        b"time,x\n0,\"1.5e2\"\n1,\"2\"\n",  // quoted scientific
        b"time,x\n 0 , 1 \n 1 , 2 \n",      // whitespace padding
        b"time,x\n0,-1.5\n1,2.25\n",        // signed / fractional
        b"time,x\n0,1e3\n1,2E-2\n",         // scientific notation
        b"y,time\n1,0\n2,1\n",              // time not first column
        b"time,x\n0,1\n1,2\n\n",            // trailing blank line (error)
        b"time,x\n0,1,2\n",                 // too many fields (error)
        b"time,x\n0\n",                     // too few fields (error)
        b"time,x\n0,abc\n",                 // non-numeric (error)
        b"time,x\n1,1\n0,2\n",              // non-increasing time (error)
        b"time,x\n0,1\n0,2\n",              // duplicate timestamp (error)
        b"time,x\n",                        // header only (error)
        b"",                                // empty (error)
        b"time,x\n0,\"1\"suffix\n",         // chars after quote (error)
        b"time,x\n0,\"unterminated\n",      // unterminated quote (error)
        b"time,x\n0,1\r2\n",                // bare CR (error)
        b",x\n0,1\n",                       // empty header name (error)
        b"time,x,x\n0,1,2\n",               // duplicate column (error)
        b"time,x\n0,\"a\"\"b\"\n",          // escaped quote non-numeric (error)
        b"time,x\n0,nan\n",                 // non-finite value (error)
    ];
    for case in cases {
        assert_equivalent(case, "time");
    }
}

#[test]
fn matches_reference_on_generated_large_inputs() {
    // Deterministic generator exercising many rows, signs, and magnitudes.
    let mut state: u64 = 0x1234_5678_9abc_def0;
    let mut next = || {
        state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        state
    };
    for cols in 1..=4 {
        let mut text = String::from("time");
        for column in 0..cols {
            text.push_str(&format!(",c{column}"));
        }
        text.push('\n');
        for row in 0..2_000usize {
            text.push_str(&row.to_string());
            for _ in 0..cols {
                let unit = (next() >> 11) as f64 / (1u64 << 53) as f64;
                let value = unit * 2_000.0 - 1_000.0;
                text.push(',');
                text.push_str(&format!("{value:.9}"));
            }
            text.push('\n');
        }
        assert_equivalent(text.as_bytes(), "time");
        // Same content without a trailing newline must also match.
        let trimmed = text.trim_end_matches('\n');
        assert_equivalent(trimmed.as_bytes(), "time");
    }
}

#[test]
fn streaming_reader_matches_in_memory_decode() {
    let mut text = String::from("time,x,y\n");
    for row in 0..5_000usize {
        text.push_str(&format!("{row},{}.5,{}\n", row * 2, (row as f64) * -0.25));
    }
    let from_bytes = read_csv_numeric(text.as_bytes(), "time").unwrap();

    let mut rows_seen = 0usize;
    let streamed = lawsynth_data::load_csv_numeric_with_progress(
        std::io::Cursor::new(text.as_bytes()),
        "time",
        |rows| rows_seen = rows,
    )
    .unwrap();

    assert_eq!(from_bytes.fingerprint(), streamed.fingerprint());
    assert_eq!(from_bytes, streamed);
    assert_eq!(rows_seen, 5_000, "progress callback must observe every row");
}

#[test]
fn progress_callback_reports_monotonic_row_counts() {
    let text = "time,x\n0,1\n1,2\n2,3\n4,5\n";
    let mut observed = Vec::new();
    let dataset = lawsynth_data::read_csv_numeric_with_progress(text.as_bytes(), "time", |rows| {
        observed.push(rows)
    })
    .unwrap();
    assert_eq!(observed, vec![1, 2, 3, 4]);
    assert_eq!(dataset.time().len(), 4);
}
