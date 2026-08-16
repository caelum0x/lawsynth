use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_dynamics::{DiscreteProblem, discrete_transitions};

#[test]
fn transitions_have_one_fewer_row_than_source_data() {
    let x = Identifier::new("x").unwrap();
    let data = Dataset::new(TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(), [NumericColumn::new(x.clone(), vec![0.0, 1.0, 2.0])]).unwrap();
    assert_eq!(discrete_transitions(&DiscreteProblem::new(data, [x]).unwrap()).unwrap().times.len(), 2);
}
