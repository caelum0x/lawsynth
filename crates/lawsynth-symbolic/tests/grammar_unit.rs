use lawsynth_core::Identifier;
use lawsynth_symbolic::Grammar;

#[test]
fn scalar_grammar_sorts_and_deduplicates_terminals() {
    let grammar = Grammar::scalar([
        Identifier::new("z").unwrap(),
        Identifier::new("x").unwrap(),
        Identifier::new("z").unwrap(),
    ]);
    assert_eq!(
        grammar.terminals(),
        &[Identifier::new("x").unwrap(), Identifier::new("z").unwrap()]
    );
}
