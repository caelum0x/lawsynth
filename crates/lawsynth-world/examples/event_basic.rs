use lawsynth_core::Identifier;
use lawsynth_world::{Event, EventDirection, crosses_zero};

fn main() {
    let event =
        Event::new(Identifier::new("threshold").unwrap(), 1.25, EventDirection::Rising).unwrap();
    assert!(crosses_zero(-0.1, 0.2, event.direction));
    println!("event {} occurred at t={}", event.id, event.time);
}
