use std::fs;

use lawsynth_bundle::read_world;
use lawsynth_cli::run;

#[test]
fn discover_command_writes_a_simulatable_world() {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-discover-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    let csv = directory.join("growth.csv");
    let contents = (0..101)
        .map(|step| {
            let time = step as f64 * 0.01;
            format!("{time},{:.17e}", (2.0 * time).exp())
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&csv, format!("t,x\n{contents}\n")).unwrap();
    let bundle = directory.join("growth.lsworld");

    let output = run(&[
        "discover".to_owned(),
        csv.display().to_string(),
        "--time".to_owned(),
        "t".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--output".to_owned(),
        bundle.display().to_string(),
        "--trigonometric".to_owned(),
        "--savgol-window".to_owned(),
        "5".to_owned(),
        "--smooth-radius".to_owned(),
        "1".to_owned(),
        "--bootstrap".to_owned(),
        "3".to_owned(),
    ])
    .unwrap();

    assert!(output.starts_with("discovered world:"));
    assert!(read_world(&bundle).is_ok());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn discover_command_accepts_tv_regularized_differentiation() {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-tvreg-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    let csv = directory.join("growth.csv");
    let contents = (0..101)
        .map(|step| {
            let time = step as f64 * 0.01;
            format!("{time},{:.17e}", (2.0 * time).exp())
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&csv, format!("t,x\n{contents}\n")).unwrap();
    let bundle = directory.join("growth.lsworld");

    let output = run(&[
        "discover".to_owned(),
        csv.display().to_string(),
        "--time".to_owned(),
        "t".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--output".to_owned(),
        bundle.display().to_string(),
        "--tvreg-lambda".to_owned(),
        "0.001".to_owned(),
        "--tvreg-iterations".to_owned(),
        "150".to_owned(),
    ])
    .unwrap();

    assert!(output.starts_with("discovered world:"));
    assert!(read_world(&bundle).is_ok());
    fs::remove_dir_all(directory).unwrap();
}
