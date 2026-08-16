# AST serialization

The expression crate exposes text parsing and printing. The version-0.1 bundle codec uses a separate compact binary preorder encoding defined in the bundle layout specification: tag `0` followed by little-endian `f64` for a constant, tag `1` plus a u16 UTF-8 identifier for a symbol, tag `2` plus unary tag plus operand, and tag `3` plus binary tag plus left then right operands.

Unary tags are `0..4` in the order Negate, Exp, Log, Sin, Cos. Binary tags are `0..4` in the order Add, Subtract, Multiply, Divide, Power. Decoders reject unknown tags, invalid identifiers, non-finite constants, trailing bytes, and trees at or beyond depth 128. This is the only normative binary expression interchange for bundle format 0.1.
