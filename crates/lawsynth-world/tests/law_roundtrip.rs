use lawsynth_core::Identifier;
use lawsynth_world::{Event, EventDirection, crosses_zero};

#[test]
fn event_marker_preserves_typed_identity_and_direction() {
    let id = Identifier::new("threshold").unwrap();
    let event = Event::new(id.clone(), 1.25, EventDirection::Rising).unwrap();
    assert_eq!(event.id, id);
    assert!(crosses_zero(-0.01, 0.02, event.direction));
    assert!(
        Event::new(
            Identifier::new("invalid").unwrap(),
            f64::NAN,
            EventDirection::Any
        )
        .is_none()
    );
}
