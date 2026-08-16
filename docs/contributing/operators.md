# Expression and operator contributions

An operator is part of the scalar World contract, not merely a parser change.
Adding one requires consistent validation, dimensional-unit rules, expression
evaluation, feature generation where applicable, symbolic rendering, binary
bundle encoding/decoding, and tests for accepted and rejected inputs.

Current scalar expressions support finite constants and symbols, unary
`neg`, `exp`, `log`, `sin`, and `cos`, plus binary `add`, `sub`, `mul`, `div`,
and `pow`. Depth is bounded. Domain errors and non-finite results must surface
as diagnostics rather than being converted to sentinel values.

Custom operators are not currently serializable or executable through the
production bundle/CLI path. Do not add a plugin-shaped placeholder; introduce
an end-to-end capability only when its execution and persistence semantics
are specified and verified.
