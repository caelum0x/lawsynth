//! Minimal JSON emission and flat-object parsing for the control-plane transport.
//!
//! The scheduler links no serialization crate, so the HTTP layer carries its own
//! dependency-free JSON support. Response bodies are small, mostly flat objects,
//! and request bodies are shallow maps of scalar values, so a compact writer plus
//! a non-recursive object parser is sufficient while keeping the offline,
//! std-only build guarantee intact.

/// An owned JSON value rendered into compact UTF-8 text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(u64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Renders the value as canonical, dependency-free JSON text.
    pub fn render(&self) -> String {
        let mut out = String::new();
        self.write_into(&mut out);
        out
    }

    fn write_into(&self, out: &mut String) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
            Self::Number(value) => out.push_str(&value.to_string()),
            Self::String(value) => write_escaped(value, out),
            Self::Array(items) => {
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    item.write_into(out);
                }
                out.push(']');
            }
            Self::Object(fields) => {
                out.push('{');
                for (index, (name, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    write_escaped(name, out);
                    out.push(':');
                    value.write_into(out);
                }
                out.push('}');
            }
        }
    }
}

fn write_escaped(value: &str, out: &mut String) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

/// A scalar value decoded from a flat JSON request body.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonScalar {
    String(String),
    Number(u64),
    Bool(bool),
    Null,
}

/// A decoded flat JSON object. Control-plane request bodies are shallow maps of
/// scalar values, so nested arrays and objects are intentionally rejected rather
/// than parsed: the transport never needs them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JsonObject {
    fields: Vec<(String, JsonScalar)>,
}

impl JsonObject {
    /// Returns the string value bound to `key`, if present and a string.
    pub fn string(&self, key: &str) -> Option<&str> {
        self.fields.iter().find_map(|(name, value)| match value {
            JsonScalar::String(text) if name == key => Some(text.as_str()),
            _ => None,
        })
    }

    /// Returns the unsigned number bound to `key`, if present and a number.
    pub fn number(&self, key: &str) -> Option<u64> {
        self.fields.iter().find_map(|(name, value)| match value {
            JsonScalar::Number(number) if name == key => Some(*number),
            _ => None,
        })
    }
}

/// Parses a flat JSON object of scalar values, rejecting anything nested.
///
/// The transport only accepts shallow control-plane bodies, so this parser is
/// deliberately small: it fails fast on nested structures rather than silently
/// accepting shapes the transport does not model.
pub fn parse_object(input: &str) -> Result<JsonObject, &'static str> {
    let mut parser = Parser { bytes: input.as_bytes(), position: 0 };
    parser.skip_whitespace();
    parser.expect(b'{')?;
    let mut fields = Vec::new();
    parser.skip_whitespace();
    if parser.peek() == Some(b'}') {
        parser.position += 1;
        parser.skip_whitespace();
        return parser.finish(fields);
    }
    loop {
        parser.skip_whitespace();
        let key = parser.parse_string()?;
        parser.skip_whitespace();
        parser.expect(b':')?;
        parser.skip_whitespace();
        let value = parser.parse_scalar()?;
        fields.push((key, value));
        parser.skip_whitespace();
        match parser.next() {
            Some(b',') => continue,
            Some(b'}') => break,
            _ => return Err("expected ',' or '}'"),
        }
    }
    parser.skip_whitespace();
    parser.finish(fields)
}

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Parser<'_> {
    fn finish(&self, fields: Vec<(String, JsonScalar)>) -> Result<JsonObject, &'static str> {
        if self.position == self.bytes.len() {
            Ok(JsonObject { fields })
        } else {
            Err("trailing bytes after JSON object")
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let byte = self.peek();
        if byte.is_some() {
            self.position += 1;
        }
        byte
    }

    fn expect(&mut self, byte: u8) -> Result<(), &'static str> {
        if self.next() == Some(byte) { Ok(()) } else { Err("unexpected token") }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.position += 1;
        }
    }

    fn parse_scalar(&mut self) -> Result<JsonScalar, &'static str> {
        match self.peek() {
            Some(b'"') => Ok(JsonScalar::String(self.parse_string()?)),
            Some(b't') => {
                self.consume_literal("true")?;
                Ok(JsonScalar::Bool(true))
            }
            Some(b'f') => {
                self.consume_literal("false")?;
                Ok(JsonScalar::Bool(false))
            }
            Some(b'n') => {
                self.consume_literal("null")?;
                Ok(JsonScalar::Null)
            }
            Some(byte) if byte.is_ascii_digit() => self.parse_number(),
            _ => Err("unsupported or nested JSON value"),
        }
    }

    fn consume_literal(&mut self, literal: &str) -> Result<(), &'static str> {
        for expected in literal.bytes() {
            if self.next() != Some(expected) {
                return Err("invalid JSON literal");
            }
        }
        Ok(())
    }

    fn parse_number(&mut self) -> Result<JsonScalar, &'static str> {
        let start = self.position;
        while matches!(self.peek(), Some(byte) if byte.is_ascii_digit()) {
            self.position += 1;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.position])
            .map_err(|_| "invalid number encoding")?;
        text.parse::<u64>().map(JsonScalar::Number).map_err(|_| "number out of range")
    }

    fn parse_string(&mut self) -> Result<String, &'static str> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            match self.next() {
                None => return Err("unterminated string"),
                Some(b'"') => return Ok(out),
                Some(b'\\') => match self.next() {
                    Some(b'"') => out.push('"'),
                    Some(b'\\') => out.push('\\'),
                    Some(b'/') => out.push('/'),
                    Some(b'n') => out.push('\n'),
                    Some(b'r') => out.push('\r'),
                    Some(b't') => out.push('\t'),
                    Some(b'b') => out.push('\u{0008}'),
                    Some(b'f') => out.push('\u{000c}'),
                    Some(b'u') => out.push(self.parse_unicode_escape()?),
                    _ => return Err("invalid escape sequence"),
                },
                Some(byte) if byte < 0x80 => out.push(byte as char),
                Some(byte) => {
                    // Continuation of a multi-byte UTF-8 sequence: reconstruct it
                    // by scanning ahead to the string's next control boundary.
                    let start = self.position - 1;
                    while matches!(self.peek(), Some(next) if next != b'"' && next != b'\\') {
                        self.position += 1;
                    }
                    let _ = byte;
                    let slice = std::str::from_utf8(&self.bytes[start..self.position])
                        .map_err(|_| "string is not valid UTF-8")?;
                    out.push_str(slice);
                }
            }
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, &'static str> {
        let mut code = 0u32;
        for _ in 0..4 {
            let byte = self.next().ok_or("truncated unicode escape")?;
            let digit = (byte as char).to_digit(16).ok_or("invalid unicode escape")?;
            code = code * 16 + digit;
        }
        char::from_u32(code).ok_or("invalid unicode scalar")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_nested_values_and_escapes_strings() {
        let value = Json::Object(vec![
            ("count".into(), Json::Number(2)),
            ("dry_run".into(), Json::Bool(true)),
            ("note".into(), Json::string("quote \" and \\ and \n newline")),
            ("ids".into(), Json::Array(vec![Json::string("a"), Json::Null])),
        ]);
        assert_eq!(
            value.render(),
            "{\"count\":2,\"dry_run\":true,\"note\":\"quote \\\" and \\\\ and \\n newline\",\"ids\":[\"a\",null]}"
        );
    }

    #[test]
    fn escapes_low_control_characters_as_unicode() {
        assert_eq!(Json::string("\u{0001}").render(), "\"\\u0001\"");
    }

    #[test]
    fn parses_flat_object_of_scalars() {
        let object = parse_object(
            "{\"id\":\"cpu-a\",\"cpu_millis\":500,\"memory_bytes\":4096,\"disk_bytes\":0}",
        )
        .unwrap();
        assert_eq!(object.string("id"), Some("cpu-a"));
        assert_eq!(object.number("cpu_millis"), Some(500));
        assert_eq!(object.number("disk_bytes"), Some(0));
        assert_eq!(object.number("missing"), None);
    }

    #[test]
    fn parses_escaped_string_values() {
        let object = parse_object("{\"reason\":\"quote \\\" tab \\t done\"}").unwrap();
        assert_eq!(object.string("reason"), Some("quote \" tab \t done"));
    }

    #[test]
    fn accepts_an_empty_object() {
        assert_eq!(parse_object("{}").unwrap(), JsonObject::default());
    }

    #[test]
    fn rejects_nested_values_and_trailing_bytes() {
        assert!(parse_object("{\"a\":{\"b\":1}}").is_err());
        assert!(parse_object("{\"a\":[1]}").is_err());
        assert!(parse_object("{\"a\":1} extra").is_err());
        assert!(parse_object("not json").is_err());
    }
}
