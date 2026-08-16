# Compatibility

Version `0.1.0` is a closed wire contract: writer and reader agree on exact manifest bytes, stored ZIP structure, entry names, and `lawsynth-world-binary-v1`. A compatible reader must reject unknown manifest bytes rather than guessing an interpretation. A compatible writer must not add entries to a 0.1 artifact because checksum validation requires the checksum file to account for every non-checksum entry.

The archive is portable across systems that preserve bytes. Lexical ordering is byte-string ordering supplied by Rust `BTreeMap`; numeric binary fields are little-endian. Floating-point payload compatibility follows IEEE-754 binary64. No promise is made for non-IEEE hosts.
