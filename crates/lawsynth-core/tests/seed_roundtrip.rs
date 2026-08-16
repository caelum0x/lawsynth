use lawsynth_core::Seed;

#[test]
fn derived_seeds_and_rng_sequences_are_reproducible_and_namespaced() {
    let root = Seed::new(42);
    let discovery = root.derive("discovery");
    assert_eq!(discovery, root.derive("discovery"));
    assert_ne!(discovery, root.derive("bootstrap"));

    let mut first = discovery.rng();
    let mut second = discovery.rng();
    let first_values = (0..8).map(|_| first.next_u64()).collect::<Vec<_>>();
    let second_values = (0..8).map(|_| second.next_u64()).collect::<Vec<_>>();
    assert_eq!(first_values, second_values);
    assert!(first.next_f64() >= 0.0 && first.next_f64() < 1.0);
}
