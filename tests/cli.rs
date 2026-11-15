use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_linen"))
        .args(arguments)
        .output()
        .expect("the linen binary should run")
}

fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).expect("stderr should be UTF-8")
}

struct TempSource {
    path: PathBuf,
}

impl TempSource {
    fn new(name: &str, contents: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("linen-stage-1-{}-{name}.ln", std::process::id()));
        fs::write(&path, contents).expect("the temporary source should be written");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempSource {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[test]
fn no_arguments_prints_help() {
    let output = run(&[]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("Usage:\n  linen --help"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn help_flags_print_help() {
    for flag in ["-h", "--help"] {
        let output = run(&[flag]);

        assert!(output.status.success(), "{flag} should succeed");
        assert!(stdout(&output).contains("linen "));
        assert!(stdout(&output).contains("  linen --version"));
        assert!(stdout(&output).contains("  linen check <input.ln>"));
        assert!(stderr(&output).is_empty());
    }
}

#[test]
fn version_flags_print_the_package_version() {
    for flag in ["-V", "--version"] {
        let output = run(&[flag]);

        assert!(output.status.success(), "{flag} should succeed");
        assert_eq!(
            stdout(&output),
            format!("linen {}\n", env!("CARGO_PKG_VERSION"))
        );
        assert!(stderr(&output).is_empty());
    }
}

#[test]
fn unsupported_arguments_are_a_usage_error() {
    let output = run(&["check"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).is_empty());
    assert!(
        stderr(&output).starts_with("error: expected --help, --version, or check <input.ln>\n")
    );
}

#[test]
fn spec_6_1_check_accepts_valid_stage_1_syntax_without_output() {
    let source = TempSource::new("valid", include_str!("../fixtures/valid/stage-01-beep.ln"));
    let output = run(&["check", source.path().to_str().expect("UTF-8 path")]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());
}

#[test]
fn spec_6_1_check_does_not_run_stage_2_semantic_checks() {
    let source = TempSource::new(
        "semantics-deferred",
        include_str!("../fixtures/valid/stage-01-semantics-deferred.ln"),
    );
    let output = run(&["check", source.path().to_str().expect("UTF-8 path")]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());
}

#[test]
fn spec_6_1_syntax_errors_use_stderr_exit_1_and_a_source_location() {
    let source = TempSource::new(
        "unknown-character",
        include_str!("../fixtures/invalid/stage-01-unknown-character.ln"),
    );
    let output = run(&["check", source.path().to_str().expect("UTF-8 path")]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("error[E1000]"));
    assert!(stderr(&output).contains(":1:11"));
}

#[test]
fn spec_6_1_source_file_errors_use_exit_2_and_e5000() {
    let path =
        std::env::temp_dir().join(format!("linen-stage-1-{}-missing.ln", std::process::id()));
    let _ = fs::remove_file(&path);
    let output = run(&["check", path.to_str().expect("UTF-8 path")]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("error[E5000]"));
}

#[test]
fn spec_6_1_invalid_utf8_is_a_file_error_not_a_panic() {
    let source = TempSource::new("invalid-utf8", "placeholder");
    fs::write(source.path(), [0xff, 0xfe]).expect("invalid UTF-8 should be written");
    let output = run(&["check", source.path().to_str().expect("UTF-8 path")]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("error[E5000]"));
}

#[test]
fn spec_6_1_check_rejects_missing_and_extra_arguments() {
    for arguments in [vec!["check"], vec!["check", "input.ln", "extra"]] {
        let output = run(&arguments);

        assert_eq!(output.status.code(), Some(2));
        assert!(stdout(&output).is_empty());
        assert!(stderr(&output).contains("check <input.ln>"));
    }
}
