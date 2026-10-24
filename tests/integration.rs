use std::env;
use std::process::{Command, Stdio};

fn kernyx_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(
        env::var("CARGO_BIN_EXE_kernyx")
            .expect("CARGO_BIN_EXE_kernyx not set; run via `cargo test`"),
    )
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
}

fn run(args: &[&str]) -> Command {
    let mut cmd = Command::new(kernyx_bin());
    cmd.current_dir(repo_root());
    cmd.args(args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd
}

fn run_string(args: &[&str]) -> (i32, String, String) {
    let output = run(args).output().expect("failed to spawn kernyx");
    let status = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (status, stdout, stderr)
}

fn example(name: &str) -> String {
    repo_root()
        .join("examples")
        .join(name)
        .to_string_lossy()
        .to_string()
}

#[test]
fn version_and_help() {
    let (status, stdout, _) = run_string(&["--version"]);
    assert_eq!(status, 0);
    assert!(stdout.contains("kernyx 0.1.0"));

    let (status, stdout, _) = run_string(&["help"]);
    assert_eq!(status, 0);
    assert!(stdout.contains("USAGE:"));
    assert!(stdout.contains("run <file>.knx"));
}

#[test]
fn nonexistent_file_errors() {
    let (status, _, stderr) = run_string(&["run", "nope.knx"]);
    assert_ne!(status, 0);
    assert!(!stderr.is_empty());
}

#[test]
fn run_hello_world() {
    let (status, stdout, stderr) = run_string(&["run", &example("hello.knx")]);
    assert_eq!(status, 0, "stderr was: {}", stderr);
    assert_eq!(stdout, "yo, world\n");
    assert!(stderr.is_empty(), "unexpected stderr: {}", stderr);
}

#[test]
fn check_hello_world() {
    let (status, stdout, stderr) = run_string(&["check", &example("hello.knx")]);
    assert_eq!(status, 0, "stderr was: {}", stderr);
    assert!(stdout.contains("no errors"));
    assert!(stderr.is_empty());
}

#[test]
fn run_arithmetic() {
    let (status, stdout, stderr) = run_string(&["run", &example("arithmetic.knx")]);
    assert_eq!(status, 0, "stderr was: {}", stderr);
    let expected = "14\n20\n5\n3\n1\ntrue\nfalse\nfalse\ntrue\n";
    assert_eq!(stdout, expected);
}

#[test]
fn run_functions() {
    let (status, stdout, stderr) = run_string(&["run", &example("functions.knx")]);
    assert_eq!(status, 0, "stderr was: {}", stderr);
    assert_eq!(stdout, "14\n");
}

#[test]
fn run_error_programs_do_not_execute() {
    let cases = &[
        "errors/bad_type.knx",
        "errors/bad_return.knx",
        "errors/bad_condition.knx",
        "errors/bad_arg.knx",
    ];
    for case in cases {
        let (status, _, stderr) = run_string(&["run", &example(case)]);
        assert_ne!(status, 0, "{} should fail", case);
        assert!(!stderr.is_empty(), "{} should have a diagnostic", case);
    }
}

#[test]
fn runtime_division_by_zero_errors() {
    let (status, _, stderr) = run_string(&["run", &example("errors/div_by_zero.knx")]);
    assert_ne!(status, 0);
    assert!(stderr.contains("division by zero"));
}

#[test]
fn check_reports_type_error_without_executing() {
    let (status, _, stderr) = run_string(&["check", &example("errors/bad_type.knx")]);
    assert_ne!(status, 0);
    assert!(stderr.contains("KX0203"));
    assert!(stderr.contains("bad_type.knx"));
}
