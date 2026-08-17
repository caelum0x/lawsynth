//! RFC-4180-style delimited numeric ingestion for CSV and TSV observations.
//!
//! The loader is a single-pass streaming state machine: it parses fields
//! directly into preallocated column vectors without materializing an
//! intermediate `Vec<Vec<String>>` of per-cell heap strings and without a
//! second numeric-parsing pass. Unquoted fields (the overwhelmingly common
//! case for numeric data) are parsed straight from the input byte slice with no
//! per-cell copy. The same core drives both the in-memory `&[u8]` entry points
//! and the [`std::io::Read`]-based streaming loaders, and both accept an
//! optional progress callback invoked with the cumulative row count.

use std::io::{BufReader, Read};

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

/// Decodes a UTF-8 CSV file, reporting cumulative rows parsed to `progress`.
///
/// The callback receives the running count of complete data rows and lets a
/// caller surface progress on large files. Default decoding behavior and output
/// are byte-for-byte identical to [`read_csv_numeric`].
pub fn read_csv_numeric_with_progress<F: FnMut(usize)>(
    bytes: &[u8],
    time_column: &str,
    progress: F,
) -> Result<Dataset, DataError> {
    read_delimited_numeric_with_progress(bytes, b',', time_column, progress)
}

/// Decodes a UTF-8 TSV file, reporting cumulative rows parsed to `progress`.
pub fn read_tsv_numeric_with_progress<F: FnMut(usize)>(
    bytes: &[u8],
    time_column: &str,
    progress: F,
) -> Result<Dataset, DataError> {
    read_delimited_numeric_with_progress(bytes, b'\t', time_column, progress)
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
    read_delimited_numeric_with_progress(bytes, delimiter, time_column, |_| {})
}

/// Decodes delimited numeric text, reporting cumulative rows to `progress`.
///
/// Identical output to [`read_delimited_numeric`]; the additional callback is
/// invoked once per completed data row with the running row count so callers
/// can drive progress reporting over large inputs.
pub fn read_delimited_numeric_with_progress<F: FnMut(usize)>(
    bytes: &[u8],
    delimiter: u8,
    time_column: &str,
    progress: F,
) -> Result<Dataset, DataError> {
    check_delimiter(delimiter)?;
    // Validate UTF-8 up front so field-level decoding is infallible and the
    // error surface matches the historical whole-input decode.
    std::str::from_utf8(bytes).map_err(|_| delimited("input is not valid UTF-8"))?;
    let mut loader = Loader::new(delimiter, time_column, estimate_rows(bytes), progress);
    loader.push_bytes(bytes)?;
    loader.finish()
}

/// Streams a CSV source through a buffered reader without loading a `String`.
pub fn load_csv_numeric<R: Read>(reader: R, time_column: &str) -> Result<Dataset, DataError> {
    load_delimited_numeric_with_progress(reader, b',', time_column, |_| {})
}

/// Streams a CSV source, reporting cumulative rows parsed to `progress`.
pub fn load_csv_numeric_with_progress<R: Read, F: FnMut(usize)>(
    reader: R,
    time_column: &str,
    progress: F,
) -> Result<Dataset, DataError> {
    load_delimited_numeric_with_progress(reader, b',', time_column, progress)
}

/// Streams a delimited numeric source through a buffered reader.
///
/// The source is consumed in bounded chunks and parsed incrementally, so peak
/// memory tracks the resulting columns rather than the raw text. Output is
/// identical to [`read_delimited_numeric`] for well-formed UTF-8 input.
pub fn load_delimited_numeric<R: Read>(
    reader: R,
    delimiter: u8,
    time_column: &str,
) -> Result<Dataset, DataError> {
    load_delimited_numeric_with_progress(reader, delimiter, time_column, |_| {})
}

/// Streams delimited numeric text, reporting cumulative rows to `progress`.
pub fn load_delimited_numeric_with_progress<R: Read, F: FnMut(usize)>(
    reader: R,
    delimiter: u8,
    time_column: &str,
    progress: F,
) -> Result<Dataset, DataError> {
    check_delimiter(delimiter)?;
    let mut loader = Loader::new(delimiter, time_column, 0, progress);
    let mut reader = BufReader::new(reader);
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let read =
            reader.read(&mut chunk).map_err(|error| delimited(format!("read failed: {error}")))?;
        if read == 0 {
            break;
        }
        loader.push_bytes(&chunk[..read])?;
    }
    loader.finish()
}

fn check_delimiter(delimiter: u8) -> Result<(), DataError> {
    if delimiter == b'\n' || delimiter == b'\r' || delimiter == b'"' {
        return Err(delimited("delimiter must not be a quote or newline"));
    }
    Ok(())
}

/// Estimates the row count from the header line width to size column vectors.
fn estimate_rows(bytes: &[u8]) -> usize {
    let line = bytes.iter().position(|&byte| byte == b'\n').map(|index| index + 1);
    let line = line.unwrap_or_else(|| bytes.len().max(1));
    (bytes.len() / line.max(1)).max(1)
}

fn delimited(reason: impl Into<String>) -> DataError {
    DataError::Delimited(reason.into())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    /// Start or middle of an unquoted field.
    Normal,
    /// Inside a double-quoted field.
    Quoted,
    /// Saw a `"` inside a quoted field; deciding escape (`""`) versus close.
    QuoteInQuoted,
    /// A quoted field just closed; only a delimiter or newline may follow.
    AfterQuote,
    /// Saw `\r` in an unquoted context; expecting `\n`.
    CrNormal,
    /// Saw `\r` after a closing quote; expecting `\n`.
    CrAfterQuote,
}

/// Incremental delimited-record parser writing directly into numeric columns.
struct Loader<'time, F: FnMut(usize)> {
    delimiter: u8,
    time_column: &'time str,
    reserve: usize,
    progress: F,
    state: State,
    field: Vec<u8>,
    saw_content: bool,
    // Header (first record) collection.
    header_done: bool,
    header_raw: Vec<String>,
    header_ids: Vec<Identifier>,
    time_index: usize,
    ncols: usize,
    // Body accumulation.
    values: Vec<Vec<f64>>,
    col_index: usize,
    row_index: usize,
    pending_parse: Option<usize>,
}

impl<'time, F: FnMut(usize)> Loader<'time, F> {
    fn new(delimiter: u8, time_column: &'time str, reserve: usize, progress: F) -> Self {
        Self {
            delimiter,
            time_column,
            reserve,
            progress,
            state: State::Normal,
            field: Vec::with_capacity(32),
            saw_content: false,
            header_done: false,
            header_raw: Vec::new(),
            header_ids: Vec::new(),
            time_index: 0,
            ncols: 0,
            values: Vec::new(),
            col_index: 0,
            row_index: 0,
            pending_parse: None,
        }
    }

    /// Feeds a chunk of input through the state machine.
    fn push_bytes(&mut self, mut input: &[u8]) -> Result<(), DataError> {
        while !input.is_empty() {
            // Fast path: at a clean field start, consume a whole unquoted field
            // (and its terminator) directly from the input slice — no per-cell
            // copy or intermediate string.
            if self.state == State::Normal && self.field.is_empty() {
                let stop = input
                    .iter()
                    .position(|&byte| {
                        byte == self.delimiter || byte == b'\n' || byte == b'\r' || byte == b'"'
                    })
                    .unwrap_or(input.len());
                if stop == input.len() {
                    // Field runs past this chunk; buffer and wait for more.
                    self.field.extend_from_slice(input);
                    self.saw_content = true;
                    return Ok(());
                }
                let terminator = input[stop];
                let content = &input[..stop];
                if terminator == b'"' {
                    if stop == 0 {
                        // Opening quote at field start.
                        self.state = State::Quoted;
                        self.saw_content = true;
                        input = &input[1..];
                        continue;
                    }
                    // Content then a stray quote — buffer content and let the
                    // byte-level path raise the canonical error.
                    self.field.extend_from_slice(content);
                    self.saw_content = true;
                    input = &input[stop..];
                    // Fall through to slow path for the quote byte.
                } else if terminator == self.delimiter {
                    self.accept_field(content)?;
                    self.saw_content = true;
                    input = &input[stop + 1..];
                    continue;
                } else if terminator == b'\n' {
                    if stop > 0 {
                        self.saw_content = true;
                    }
                    self.accept_field(content)?;
                    self.end_record()?;
                    input = &input[stop + 1..];
                    continue;
                } else {
                    // Carriage return: buffer content, defer terminator decision.
                    self.field.extend_from_slice(content);
                    if stop > 0 {
                        self.saw_content = true;
                    }
                    self.state = State::CrNormal;
                    input = &input[stop + 1..];
                    continue;
                }
            }
            let byte = input[0];
            self.push_byte(byte)?;
            input = &input[1..];
        }
        Ok(())
    }

    /// Processes a single byte through the state machine (handles all the
    /// quoting, escaping, and newline edge cases the fast path skips).
    fn push_byte(&mut self, byte: u8) -> Result<(), DataError> {
        loop {
            match self.state {
                State::Normal => {
                    if byte == b'"' {
                        if self.field.is_empty() {
                            self.state = State::Quoted;
                            self.saw_content = true;
                        } else {
                            return Err(delimited("quote inside an unquoted field"));
                        }
                    } else if byte == self.delimiter {
                        self.end_field()?;
                        self.saw_content = true;
                    } else if byte == b'\n' {
                        self.end_field()?;
                        self.end_record()?;
                    } else if byte == b'\r' {
                        self.state = State::CrNormal;
                    } else {
                        self.field.push(byte);
                        self.saw_content = true;
                    }
                    return Ok(());
                }
                State::Quoted => {
                    if byte == b'"' {
                        self.state = State::QuoteInQuoted;
                    } else {
                        self.field.push(byte);
                    }
                    return Ok(());
                }
                State::QuoteInQuoted => {
                    if byte == b'"' {
                        self.field.push(b'"');
                        self.state = State::Quoted;
                        return Ok(());
                    }
                    // Closing quote confirmed; reprocess this byte after quote.
                    self.state = State::AfterQuote;
                    continue;
                }
                State::AfterQuote => {
                    if byte == self.delimiter {
                        self.end_field()?;
                        self.saw_content = true;
                    } else if byte == b'\n' {
                        self.end_field()?;
                        self.end_record()?;
                    } else if byte == b'\r' {
                        self.state = State::CrAfterQuote;
                    } else {
                        return Err(delimited(
                            "characters after a closing quote must be a delimiter or newline",
                        ));
                    }
                    return Ok(());
                }
                State::CrNormal => {
                    if byte == b'\n' {
                        self.end_field()?;
                        self.end_record()?;
                        return Ok(());
                    }
                    return Err(delimited("bare carriage return is not a record separator"));
                }
                State::CrAfterQuote => {
                    if byte == b'\n' {
                        self.end_field()?;
                        self.end_record()?;
                        return Ok(());
                    }
                    return Err(delimited(
                        "characters after a closing quote must be a delimiter or newline",
                    ));
                }
            }
        }
    }

    /// Completes the current scratch field, then returns to `Normal`.
    fn end_field(&mut self) -> Result<(), DataError> {
        let field = std::mem::take(&mut self.field);
        let result = self.accept_field(&field);
        self.field = field;
        self.field.clear();
        self.state = State::Normal;
        result
    }

    /// Records one completed field's content into the header or a column.
    fn accept_field(&mut self, bytes: &[u8]) -> Result<(), DataError> {
        if !self.header_done {
            let text =
                std::str::from_utf8(bytes).map_err(|_| delimited("input is not valid UTF-8"))?;
            self.header_raw.push(text.to_owned());
            return Ok(());
        }
        let column = self.col_index;
        self.col_index += 1;
        if column < self.ncols {
            match std::str::from_utf8(bytes).ok().and_then(|text| text.trim().parse::<f64>().ok()) {
                Some(value) => self.values[column].push(value),
                None => {
                    if self.pending_parse.is_none() {
                        self.pending_parse = Some(column);
                    }
                }
            }
        }
        Ok(())
    }

    /// Completes the current record: finalizes the header or validates a row.
    fn end_record(&mut self) -> Result<(), DataError> {
        if !self.header_done {
            self.finalize_header()?;
        } else {
            let count = self.col_index;
            if count != self.ncols {
                return Err(delimited(format!(
                    "record {} has {} fields; expected {}",
                    self.row_index + 2,
                    count,
                    self.ncols
                )));
            }
            if let Some(column) = self.pending_parse {
                return Err(delimited(format!(
                    "record {}, column '{}' is not a number",
                    self.row_index + 2,
                    self.header_ids[column]
                )));
            }
            self.row_index += 1;
            (self.progress)(self.row_index);
        }
        self.col_index = 0;
        self.pending_parse = None;
        self.saw_content = false;
        Ok(())
    }

    /// Validates the header record and prepares column storage.
    fn finalize_header(&mut self) -> Result<(), DataError> {
        if self.header_raw.is_empty() || self.header_raw.iter().any(|field| field.trim().is_empty())
        {
            return Err(delimited("header has an empty column name"));
        }
        let mut header_ids = Vec::with_capacity(self.header_raw.len());
        for (index, field) in self.header_raw.iter().enumerate() {
            let field = field.trim();
            let field =
                if index == 0 { field.strip_prefix('\u{feff}').unwrap_or(field) } else { field };
            let identifier = Identifier::new(field)
                .map_err(|error| delimited(format!("invalid header '{field}': {error}")))?;
            header_ids.push(identifier);
        }
        let time_index = header_ids
            .iter()
            .position(|name| name.as_str() == self.time_column)
            .ok_or_else(|| delimited(format!("missing time column '{}'", self.time_column)))?;
        self.ncols = header_ids.len();
        self.time_index = time_index;
        self.header_ids = header_ids;
        self.values = (0..self.ncols).map(|_| Vec::with_capacity(self.reserve)).collect();
        self.header_done = true;
        Ok(())
    }

    /// Flushes any trailing record and materializes the validated dataset.
    fn finish(mut self) -> Result<Dataset, DataError> {
        match self.state {
            State::Quoted => return Err(delimited("unterminated quoted field")),
            State::CrNormal => {
                return Err(delimited("bare carriage return is not a record separator"));
            }
            State::CrAfterQuote => {
                return Err(delimited(
                    "characters after a closing quote must be a delimiter or newline",
                ));
            }
            // A quote at end of input closes the field.
            State::QuoteInQuoted => self.state = State::AfterQuote,
            State::Normal | State::AfterQuote => {}
        }
        if self.saw_content || !self.field.is_empty() || self.col_index > 0 {
            self.end_field()?;
            self.end_record()?;
        }
        if !self.header_done {
            return Err(delimited("input has no header"));
        }
        let time_values = std::mem::take(&mut self.values[self.time_index]);
        let time = TimeAxis::new(time_values)?;
        let mut columns = Vec::with_capacity(self.ncols.saturating_sub(1));
        for (index, values) in self.values.drain(..).enumerate() {
            if index == self.time_index {
                continue;
            }
            columns.push(NumericColumn::new(self.header_ids[index].clone(), values));
        }
        Dataset::new(time, columns)
    }
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
