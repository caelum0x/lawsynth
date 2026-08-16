# Identifiers

Every public handle in World IR is a `lawsynth_core::Identifier`. It is a non-empty ASCII string whose first byte is not a digit. Each character must be an ASCII letter, ASCII digit, `_`, or `-`. Thus `x`, `population_2`, and `supply-demand` are valid; `2x`, `x y`, `/x`, non-ASCII names, and the empty string are invalid.

Identifiers compare and order by their exact byte strings. No Unicode normalization, case folding, aliasing, or escaping is performed. The same identifier namespace is shared by variables and parameters, preventing shadowing. Bundle decoding re-validates identifiers rather than assuming bytes are trustworthy.
