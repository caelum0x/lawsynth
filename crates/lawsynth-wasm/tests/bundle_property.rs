use lawsynth_wasm::{Bundle, Event, EventDirection, Expression, World};
#[test]
fn binary_bundle_roundtrips_world_and_events_and_rejects_truncation() {
    let world = World::new(vec!["x".into()], vec![1.0], vec![Expression::parse("-0.5*x").unwrap()])
        .unwrap();
    let bundle = Bundle::new(
        world,
        vec![Event::new("zero", Expression::parse("x").unwrap(), EventDirection::Falling).unwrap()],
    )
    .unwrap();
    let encoded = bundle.encode().unwrap();
    assert_eq!(Bundle::decode(&encoded).unwrap(), bundle);
    for size in 0..encoded.len() {
        assert!(Bundle::decode(&encoded[..size]).is_err());
    }
}
