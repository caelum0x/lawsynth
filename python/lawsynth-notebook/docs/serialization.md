# Serialization

`load_json` reads only a UTF-8 JSON object from a caller-provided local path.
`canonical_json` uses sorted keys, compact separators, and rejects non-finite
numbers so an exported artifact can be compared byte-for-byte.
