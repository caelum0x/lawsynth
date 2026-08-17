//! Minimal JSON value emission for the gateway's own responses.
//!
//! The gateway links no serialization crate. Its self-generated bodies
//! (`/healthz`, error envelopes) are small flat objects, so a dependency-free
//! writer that escapes strings correctly keeps the offline, std-only build
//! guarantee intact. Upstream bodies are forwarded verbatim and never parsed.

/// An owned JSON value renderable to compact UTF-8 text.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_nested_values_and_escapes_strings() {
        let value = Json::Object(vec![
            ("count".into(), Json::Number(2)),
            ("ready".into(), Json::Bool(true)),
            ("note".into(), Json::string("quote \" and \\ and \n newline")),
        ]);
        assert_eq!(
            value.render(),
            "{\"count\":2,\"ready\":true,\"note\":\"quote \\\" and \\\\ and \\n newline\"}"
        );
    }

    #[test]
    fn escapes_low_control_characters_as_unicode() {
        assert_eq!(Json::string("\u{0001}").render(), "\"\\u0001\"");
    }
}
