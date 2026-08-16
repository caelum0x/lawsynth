use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_dynamics::ContinuousProblem;

#[test]
fn continuous_problem_keeps_declared_state_schema() {
    let x = Identifier::new("x").unwrap();
    let data = Dataset::new(TimeAxis::new(vec![0.0, 1.0]).unwrap(), [NumericColumn::new(x.clone(), vec![1.0, 2.0])]).unwrap();
    assert_eq!(ContinuousProblem::new(data, [x.clone()]).unwrap().state(), &[x]);
}
