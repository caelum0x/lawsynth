use crate::{Grammar, Population, SymbolicConfig, enumerate};

/// Initializes a bounded, reproducible population directly from a grammar.
pub fn initialize_population(grammar: &Grammar, config: &SymbolicConfig) -> Population {
    Population::new(enumerate(grammar, config))
}
