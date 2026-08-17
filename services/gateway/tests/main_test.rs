use std::process::Command;

/// Path to the compiled gateway binary, provided by Cargo to integration tests.
fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_lawsynth-gateway")
}

#[test]
fn no_arguments_prints_usage_and_exits_nonzero() {
    let output = Command::new(binary()).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("usage: lawsynth-gateway serve"), "stderr: {stderr}");
}

#[test]
fn unknown_subcommand_is_rejected() {
    let output = Command::new(binary()).arg("frobnicate").output().unwrap();
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn serve_with_missing_upstream_argument_is_rejected() {
    let output = Command::new(binary()).args(["serve", "127.0.0.1:0"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("usage"), "stderr: {stderr}");
}
