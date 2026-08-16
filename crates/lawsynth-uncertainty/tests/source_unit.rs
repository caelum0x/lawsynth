use lawsynth_uncertainty::{
    SourceKind, StructuralUncertainty, UncertaintySource, structural_score,
};

#[test]
fn structural_sources_combine_in_quadrature() {
    let uncertainty = StructuralUncertainty::new(vec![
        UncertaintySource {
            name: "missing drag correction".into(),
            kind: SourceKind::Structural,
            standard_deviation: 0.3,
        },
        UncertaintySource {
            name: "alternative constitutive law".into(),
            kind: SourceKind::Structural,
            standard_deviation: 0.4,
        },
    ])
    .unwrap();
    assert!((uncertainty.combined_standard_deviation() - 0.5).abs() < 1e-12);
    let ambiguity = structural_score(&[100.0, 101.0, 120.0]).unwrap();
    assert!(ambiguity > 0.0 && ambiguity < 1.0);
}
