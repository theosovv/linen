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
    assert!(stderr(&output).starts_with("error: expected --help or --version\n"));
}
