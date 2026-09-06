//! The version flag must report the package version, not the usage block.

use lawsynth_cli::run;

#[test]
fn version_flags_report_the_package_version() {
    let expected = format!("lawsynth {}\n", env!("CARGO_PKG_VERSION"));
    for flag in ["--version", "-V", "version"] {
        let output = run(&[flag.to_owned()]).expect("version flag should succeed");
        assert_eq!(output, expected, "flag `{flag}` should print the version");
    }
}

#[test]
fn version_output_is_a_single_clean_line() {
    let output = run(&["--version".to_owned()]).unwrap();
    // A version string, not the multi-line usage block.
    assert!(output.starts_with("lawsynth "));
    assert_eq!(output.lines().count(), 1);
    assert!(!output.contains("usage:"));
}
