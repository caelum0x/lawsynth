#[path = "../src/py_plan.rs"]
mod py_plan;

use py_plan::state_identifiers;

#[test]
fn valid_portable_state_names_are_preserved_in_input_order() {
    let cases = [
        vec!["x".to_owned()],
        vec!["state_1".to_owned(), "supply-demand".to_owned()],
        vec![
            "Z9".to_owned(),
            "_internal".to_owned(),
            "control-2".to_owned(),
        ],
    ];

    for names in cases {
        let expected = names.clone();
        let plan = state_identifiers(names).expect("portable state names must form a plan");
        assert_eq!(
            plan.iter().map(ToString::to_string).collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn empty_and_non_portable_state_plans_are_rejected() {
    assert_eq!(
        state_identifiers(Vec::<String>::new()).unwrap_err(),
        "at least one state is required"
    );
    assert!(state_identifiers(["x".to_owned(), "not valid".to_owned()]).is_err());
    assert!(state_identifiers(["1st".to_owned()]).is_err());
}
