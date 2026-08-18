//! Contract-level integration tests: determinism, the "controls are never
//! differentiated / never predicted" guarantee, and boundary error paths.

mod common;

use common::{id, integrate_oscillator, oscillator_dataset};
use lawsynth_control::{ControlConfig, ControlError, ControlSpec, discover_controlled};
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

/// Identical `(dataset, spec, config)` inputs produce bit-identical models.
#[test]
fn discovery_is_bit_identical_across_runs() {
    let dataset = oscillator_dataset();
    let spec = ControlSpec::new([id("x"), id("y")], [id("u")]).unwrap();
    let config = ControlConfig::default();

    let first = discover_controlled(&dataset, &spec, &config).unwrap();
    let second = discover_controlled(&dataset, &spec, &config).unwrap();

    // Structural equality plus explicit bit-for-bit coefficient comparison.
    assert_eq!(first, second);
    for (a, b) in first.equations.iter().zip(&second.equations) {
        assert_eq!(a.coefficients.to_bits_vec(), b.coefficients.to_bits_vec());
        assert_eq!(a.residual_sum_squares.to_bits(), b.residual_sum_squares.to_bits());
    }
}

trait BitsVec {
    fn to_bits_vec(&self) -> Vec<u64>;
}

impl BitsVec for Vec<f64> {
    fn to_bits_vec(&self) -> Vec<u64> {
        self.iter().map(|value| value.to_bits()).collect()
    }
}

/// The model predicts exactly the states and never any control: one equation per
/// state, and every control appears only as a library input.
#[test]
fn controls_are_inputs_never_predicted() {
    let dataset = oscillator_dataset();
    let spec = ControlSpec::new([id("x"), id("y")], [id("u")]).unwrap();
    let model = discover_controlled(&dataset, &spec, &ControlConfig::default()).unwrap();

    assert_eq!(model.equations.len(), model.states.len());
    for equation in &model.equations {
        assert!(model.states.contains(&equation.state));
        assert!(!model.controls.contains(&equation.state));
    }
    assert!(model.equation(&id("u")).is_none());
    // The control still participates: at least one library term references `u`.
    assert!(model.library_terms.iter().any(|term| term.contains('u')));
}

/// Controls are never differentiated: perturbing ONLY the control column leaves
/// the state-derivative targets — and hence every fitted coefficient — unchanged.
///
/// This is the operational proof of the "never differentiated" contract: if the
/// control were differentiated into a target, changing it would move the fit.
#[test]
fn perturbing_control_does_not_change_state_derivative_targets() {
    let (time, xs, ys, us) = integrate_oscillator(2000, 0.005);
    let spec = ControlSpec::new([id("x"), id("y")], [id("u")]).unwrap();

    // Baseline model.
    let baseline = Dataset::new(
        TimeAxis::new(time.clone()).unwrap(),
        [
            NumericColumn::new(id("x"), xs.clone()),
            NumericColumn::new(id("y"), ys.clone()),
            NumericColumn::new(id("u"), us.clone()),
        ],
    )
    .unwrap();
    let baseline_model = discover_controlled(&baseline, &spec, &ControlConfig::default()).unwrap();

    // Replace the control column with a completely different signal, keeping the
    // states (and therefore ẋ) untouched.
    let scrambled_u = us.iter().enumerate().map(|(i, u)| u + (i as f64).cos()).collect::<Vec<_>>();
    let scrambled = Dataset::new(
        TimeAxis::new(time).unwrap(),
        [
            NumericColumn::new(id("x"), xs),
            NumericColumn::new(id("y"), ys),
            NumericColumn::new(id("u"), scrambled_u),
        ],
    )
    .unwrap();

    // Compare the state-derivative targets directly by differentiating the state
    // columns of both datasets — they must be byte-identical because the control
    // never enters differentiation.
    use lawsynth_differentiate::differentiate_series;
    let t = baseline.time().values();
    for state in ["x", "y"] {
        let a = differentiate_series(t, &baseline.columns()[&id(state)].values).unwrap();
        let b = differentiate_series(t, &scrambled.columns()[&id(state)].values).unwrap();
        assert_eq!(a, b, "state derivative target for {state} changed with the control");
    }

    // The scrambled control does change the FIT (because the library sees it), so
    // the two models differ — which is exactly why the control matters as an
    // input while never being differentiated.
    let scrambled_model =
        discover_controlled(&scrambled, &spec, &ControlConfig::default()).unwrap();
    assert_ne!(baseline_model, scrambled_model);
}

/// Error path: a spec with no controls is rejected at construction time.
#[test]
fn spec_rejects_empty_controls() {
    assert_eq!(ControlSpec::new([id("x")], []), Err(ControlError::NoControls));
}

/// Error path: an unknown control identifier is rejected against the dataset.
#[test]
fn discovery_rejects_unknown_identifier() {
    let dataset = oscillator_dataset();
    let spec = ControlSpec::new([id("x"), id("y")], [id("w")]).unwrap();
    assert_eq!(
        discover_controlled(&dataset, &spec, &ControlConfig::default()),
        Err(ControlError::UnknownIdentifier("w".into()))
    );
}

/// Error path: too few samples make differentiation impossible.
#[test]
fn discovery_rejects_too_few_samples() {
    let dataset = Dataset::new(
        TimeAxis::new(vec![0.0]).unwrap(),
        [NumericColumn::new(id("x"), vec![1.0]), NumericColumn::new(id("u"), vec![0.5])],
    )
    .unwrap();
    let spec = ControlSpec::new([id("x")], [id("u")]).unwrap();
    let error = discover_controlled(&dataset, &spec, &ControlConfig::default()).unwrap_err();
    assert!(
        matches!(error, ControlError::Differentiation(_)),
        "expected a differentiation error, got {error:?}"
    );
}
