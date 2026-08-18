//! Minimal, dependency-free JSON value model with a recursive-descent parser
//! and a compact writer.
//!
//! LawSynth's WASM bindings deliberately avoid `serde` (and every other external
//! crate) so the crate builds fully offline and produces a lean `.wasm`. This
//! module is `std`-only and deterministic: parsing and serialization never touch
//! the clock, the filesystem, or any source of randomness.

use std::fmt::Write as _;

/// Maximum nesting depth accepted by the parser. This bounds recursion so a
/// hostile, deeply nested document cannot exhaust the stack.
const MAX_DEPTH: usize = 256;

/// An in-memory JSON value.
///
/// Objects preserve insertion order and are stored as a `Vec` of pairs: worlds
/// are small, lookups are linear, and order-preservation keeps output stable and
/// diff-friendly.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// Look up a key in an object, returning `None` for non-objects or misses.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Interpret the value as a finite `f64`, or `None`.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(value) if value.is_finite() => Some(*value),
            _ => None,
        }
    }

    /// Interpret the value as a string slice, or `None`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(value) => Some(value.as_str()),
            _ => None,
        }
    }

    /// Interpret the value as an array slice, or `None`.
    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(values) => Some(values.as_slice()),
            _ => None,
        }
    }

    /// Serialize the value to a compact JSON string.
    pub fn to_json_string(&self) -> String {
        let mut out = String::new();
        write_value(&mut out, self);
        out
    }
}

/// Parse a UTF-8 JSON document into a [`Json`] value.
pub fn parse(source: &str) -> Result<Json, String> {
    let mut parser = Parser { chars: source.as_bytes(), at: 0, chars_str: source };
    parser.skip_ws();
    let value = parser.parse_value(0)?;
    parser.skip_ws();
    if parser.at != parser.chars.len() {
        return Err("trailing characters after JSON value".to_string());
    }
    Ok(value)
}

struct Parser<'a> {
    chars: &'a [u8],
    chars_str: &'a str,
    at: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.chars.get(self.at).copied()
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.at += 1;
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<Json, String> {
        if depth > MAX_DEPTH {
            return Err("maximum JSON nesting depth exceeded".to_string());
        }
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(depth),
            Some(b'[') => self.parse_array(depth),
            Some(b'"') => self.parse_string().map(Json::Str),
            Some(b't') | Some(b'f') => self.parse_bool(),
            Some(b'n') => self.parse_null(),
            Some(b'-') | Some(b'0'..=b'9') => self.parse_number(),
            _ => Err("unexpected token while parsing JSON value".to_string()),
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<Json, String> {
        self.at += 1; // consume '{'
        let mut entries: Vec<(String, Json)> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.at += 1;
            return Ok(Json::Obj(entries));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err("expected string key in JSON object".to_string());
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err("expected ':' in JSON object".to_string());
            }
            self.at += 1;
            let value = self.parse_value(depth + 1)?;
            entries.push((key, value));
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.at += 1;
                }
                Some(b'}') => {
                    self.at += 1;
                    return Ok(Json::Obj(entries));
                }
                _ => return Err("expected ',' or '}' in JSON object".to_string()),
            }
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<Json, String> {
        self.at += 1; // consume '['
        let mut values: Vec<Json> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.at += 1;
            return Ok(Json::Arr(values));
        }
        loop {
            let value = self.parse_value(depth + 1)?;
            values.push(value);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.at += 1;
                }
                Some(b']') => {
                    self.at += 1;
                    return Ok(Json::Arr(values));
                }
                _ => return Err("expected ',' or ']' in JSON array".to_string()),
            }
        }
    }

    fn parse_bool(&mut self) -> Result<Json, String> {
        if self.chars_str[self.at..].starts_with("true") {
            self.at += 4;
            Ok(Json::Bool(true))
        } else if self.chars_str[self.at..].starts_with("false") {
            self.at += 5;
            Ok(Json::Bool(false))
        } else {
            Err("invalid literal in JSON".to_string())
        }
    }

    fn parse_null(&mut self) -> Result<Json, String> {
        if self.chars_str[self.at..].starts_with("null") {
            self.at += 4;
            Ok(Json::Null)
        } else {
            Err("invalid literal in JSON".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<Json, String> {
        let start = self.at;
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        while let Some(b) = self.peek() {
            if b.is_ascii_digit() || b == b'.' || b == b'e' || b == b'E' || b == b'+' || b == b'-' {
                self.at += 1;
            } else {
                break;
            }
        }
        let text = &self.chars_str[start..self.at];
        let value: f64 = text.parse().map_err(|_| format!("invalid JSON number {text}"))?;
        if !value.is_finite() {
            return Err("JSON number must be finite".to_string());
        }
        Ok(Json::Num(value))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.at += 1; // consume opening quote
        let mut out = String::new();
        loop {
            let byte = self.peek().ok_or("unterminated JSON string")?;
            match byte {
                b'"' => {
                    self.at += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.at += 1;
                    let escape = self.peek().ok_or("unterminated escape in JSON string")?;
                    self.at += 1;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let code = self.parse_hex4()?;
                            self.push_unicode(code, &mut out)?;
                        }
                        _ => return Err("invalid escape in JSON string".to_string()),
                    }
                }
                _ => {
                    // Copy one whole UTF-8 scalar so multi-byte characters survive.
                    let rest = &self.chars_str[self.at..];
                    let ch = rest.chars().next().ok_or("invalid UTF-8 in JSON string")?;
                    out.push(ch);
                    self.at += ch.len_utf8();
                }
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u32, String> {
        let end = self.at + 4;
        let slice =
            self.chars_str.get(self.at..end).ok_or("truncated \\u escape in JSON string")?;
        let code = u32::from_str_radix(slice, 16).map_err(|_| "invalid \\u escape".to_string())?;
        self.at = end;
        Ok(code)
    }

    fn push_unicode(&mut self, high: u32, out: &mut String) -> Result<(), String> {
        // Combine a UTF-16 surrogate pair when present; otherwise decode directly.
        if (0xD800..=0xDBFF).contains(&high) {
            if self.chars_str[self.at..].starts_with("\\u") {
                self.at += 2;
                let low = self.parse_hex4()?;
                let combined = 0x10000 + (((high - 0xD800) << 10) | (low.wrapping_sub(0xDC00)));
                let ch = char::from_u32(combined).ok_or("invalid surrogate pair")?;
                out.push(ch);
                return Ok(());
            }
            return Err("unpaired UTF-16 surrogate in JSON string".to_string());
        }
        let ch = char::from_u32(high).ok_or("invalid unicode scalar in JSON string")?;
        out.push(ch);
        Ok(())
    }
}

fn write_value(out: &mut String, value: &Json) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(true) => out.push_str("true"),
        Json::Bool(false) => out.push_str("false"),
        Json::Num(number) => write_number(out, *number),
        Json::Str(text) => write_string(out, text),
        Json::Arr(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_value(out, item);
            }
            out.push(']');
        }
        Json::Obj(entries) => {
            out.push('{');
            for (index, (key, item)) in entries.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_string(out, key);
                out.push(':');
                write_value(out, item);
            }
            out.push('}');
        }
    }
}

fn write_number(out: &mut String, number: f64) {
    // Non-finite values never reach serialization (all inputs are validated
    // finite), but guard anyway so we never emit invalid JSON.
    if number.is_finite() {
        let _ = write!(out, "{number}");
    } else {
        out.push('0');
    }
}

fn write_string(out: &mut String, text: &str) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}
