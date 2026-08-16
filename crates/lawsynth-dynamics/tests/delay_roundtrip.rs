use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_dynamics::DelayedProblem;

#[test]
fn delayed_problem_preserves_lag_alignment() {
    let x = Identifier::new("x").unwrap();
    let data = Dataset::new(TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(), [NumericColumn::new(x.clone(), vec![1.0, 2.0, 3.0])]).unwrap();
    assert_eq!(DelayedProblem::new(data, [x], 1).unwrap().samples().current, vec![vec![2.0], vec![3.0]]);
}
