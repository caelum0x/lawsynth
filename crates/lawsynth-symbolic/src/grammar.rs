use lawsynth_core::Identifier;

/// The typed scalar terminals allowed in a symbolic search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grammar {
    terminals: Vec<Identifier>,
}

impl Grammar {
    pub fn scalar(terminals: impl IntoIterator<Item = Identifier>) -> Self {
        let mut terminals = terminals.into_iter().collect::<Vec<_>>();
        terminals.sort();
        terminals.dedup();
        Self { terminals }
    }

    pub fn terminals(&self) -> &[Identifier] {
        &self.terminals
    }
}
