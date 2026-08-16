# Equations

LawSynth uses a small, deterministic scalar expression IR for executable laws and discovered candidates. Expressions contain finite constants, identifiers, unary operators, and binary operators. The parser, printer, evaluator, derivative routine, simplifier, and structural fingerprint all operate on this same tree.

The supported language stays deliberately narrow so a model can pass through Rust, Python bindings, discovery, simulation, and bundles without changing meaning. Keep transformations explicit and preserve the printed expression with the selected result.
