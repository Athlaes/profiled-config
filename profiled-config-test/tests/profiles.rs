use std::process::Command;

fn run_profiled_config(args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_profiled_config_test"))
        .args(args)
        .output()
        .expect("failed to execute profiled_config_test");

    assert!(
        output.status.success(),
        "profiled_config_test failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn loads_default_configuration_without_profile() {
    run_profiled_config(&[]);
}

#[test]
fn loads_dev_profile_configuration() {
    run_profiled_config(&["--profiles", "dev"]);
}

#[test]
fn loads_multiple_overrides_cli_args() {
    run_profiled_config(&[
        "--profiles",
        "overrided",
        "--overrides",
        "test.overrided=true",
        "--overrides",
        "test.value=overrided_value",
    ]);
}
