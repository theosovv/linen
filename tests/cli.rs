use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEMP_ID: AtomicUsize = AtomicUsize::new(0);

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
    directory: PathBuf,
    path: PathBuf,
}

impl TempSource {
    fn new(name: &str, contents: &str) -> Self {
        let unique = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "linen-stage-1a-{}-{unique}-{name}",
            std::process::id()
        ));
        let path = directory.join(format!("{name}.ln"));
        fs::create_dir_all(&directory).expect("the temporary directory should be created");
        fs::write(&path, contents).expect("the temporary source should be written");
        Self { directory, path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempSource {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
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
        assert!(stdout(&output).contains("  linen run <input.ln> --output <output.wav>"));
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
        stderr(&output).starts_with(
            "error: expected --help, --version, check <input.ln>, or run <input.ln> --output <output.wav>\n"
        )
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
fn spec_9_stage_2_check_runs_name_and_type_semantics() {
    let source = TempSource::new(
        "semantics-deferred",
        include_str!("../fixtures/valid/stage-01-semantics-deferred.ln"),
    );
    let output = run(&["check", source.path().to_str().expect("UTF-8 path")]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("error[E2001]"));
    assert!(stderr(&output).contains("semantics-deferred.ln:1:13"));
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
    assert!(stderr(&output).contains("unknown-character.ln:1:11"));
}

#[test]
fn spec_6_1_source_file_errors_use_exit_2_and_e5000() {
    let path =
        std::env::temp_dir().join(format!("linen-stage-1a-{}-missing.ln", std::process::id()));
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

#[test]
fn spec_6_1_source_diagnostics_use_only_file_name_across_parent_directories() {
    let invalid = include_str!("../fixtures/invalid/stage-01-unknown-character.ln");
    let first = TempSource::new("shared", invalid);
    let second = TempSource::new("shared", invalid);

    assert_ne!(first.path().parent(), second.path().parent());

    let first_output = run(&["check", first.path().to_str().expect("UTF-8 path")]);
    let second_output = run(&["check", second.path().to_str().expect("UTF-8 path")]);

    assert_eq!(first_output.status.code(), Some(1));
    assert_eq!(second_output.status.code(), Some(1));
    assert!(stdout(&first_output).is_empty());
    assert!(stdout(&second_output).is_empty());
    assert_eq!(stderr(&first_output), stderr(&second_output));
    assert!(stderr(&first_output).contains("shared.ln:1:11"));

    for source in [&first, &second] {
        let parent = source
            .path()
            .parent()
            .expect("the source has a parent directory")
            .to_str()
            .expect("the temporary parent is UTF-8");
        assert!(!stderr(&first_output).contains(parent));
    }
}

#[test]
fn spec_5_11_independent_cli_processes_have_identical_results() {
    let sources = [
        TempSource::new(
            "repeat-valid",
            include_str!("../fixtures/valid/stage-01-beep.ln"),
        ),
        TempSource::new(
            "repeat-invalid",
            include_str!("../fixtures/invalid/stage-01-unknown-character.ln"),
        ),
    ];

    for source in sources {
        let arguments = ["check", source.path().to_str().expect("UTF-8 path")];
        let first = run(&arguments);
        let second = run(&arguments);

        assert_eq!(first.status.code(), second.status.code());
        assert_eq!(stdout(&first), stdout(&second));
        assert_eq!(stderr(&first), stderr(&second));
    }
}

fn output_path(source: &TempSource, name: &str) -> PathBuf {
    source.directory.join(name)
}

fn run_output(source: &TempSource, name: &str, extra: &[&str]) -> (Output, PathBuf) {
    let output_path = output_path(source, name);
    let mut arguments = vec![
        "run",
        source.path().to_str().expect("UTF-8 source path"),
        "--output",
        output_path.to_str().expect("UTF-8 output path"),
    ];
    arguments.extend_from_slice(extra);
    (run(&arguments), output_path)
}

fn assert_no_temporary_files(source: &TempSource) {
    let temporary_files: Vec<_> = fs::read_dir(&source.directory)
        .expect("the temporary directory should be readable")
        .map(|entry| {
            entry
                .expect("directory entry should be readable")
                .file_name()
        })
        .filter(|name| name.to_string_lossy().ends_with(".tmp"))
        .collect();
    assert!(temporary_files.is_empty(), "leftovers: {temporary_files:?}");
}

#[test]
fn spec_6_1_run_validates_required_and_duplicate_arguments_before_io() {
    for arguments in [
        vec!["run"],
        vec!["run", "input.ln"],
        vec!["run", "--output", "out.wav"],
        vec!["run", "input.ln", "--output", "out.wav", "--unknown"],
        vec![
            "run",
            "input.ln",
            "--output",
            "out.wav",
            "--sample-rate",
            "48000",
            "--sample-rate",
            "48000",
        ],
    ] {
        let output = run(&arguments);
        assert_eq!(output.status.code(), Some(2), "arguments: {arguments:?}");
        assert!(stdout(&output).is_empty());
    }
}

#[test]
fn spec_6_4_invalid_numeric_render_options_are_e4000_exit_2() {
    for (option, value) in [
        ("--sample-rate", "7999"),
        ("--sample-rate", "384001"),
        ("--sample-rate", "abc"),
        ("--block-size", "0"),
        ("--block-size", "abc"),
        ("--max-seconds", "-1"),
        ("--max-seconds", "abc"),
    ] {
        let output = run(&["run", "missing.ln", "--output", "unused.wav", option, value]);
        assert_eq!(output.status.code(), Some(2), "{option} {value}");
        assert!(stdout(&output).is_empty());
        assert!(stderr(&output).contains("error[E4000]"), "{option} {value}");
        assert!(stderr(&output).contains(value), "{option} {value}");
    }
}

#[test]
fn spec_6_1_run_creates_beep_wav_and_prints_a_short_report() {
    let source = TempSource::new("beep", include_str!("../fixtures/valid/stage-01-beep.ln"));
    let (output, wav_path) = run_output(&source, "beep.wav", &[]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());
    assert!(stdout(&output).contains(&format!("output: {}", wav_path.display())));
    assert!(stdout(&output).contains("samples: 96000"));
    assert!(stdout(&output).contains("sample rate: 48000 Hz"));
    assert!(stdout(&output).contains("duration: 2 s"));
    assert_eq!(
        fs::metadata(&wav_path).expect("the WAV should exist").len(),
        44 + 96_000 * 2
    );
    assert_no_temporary_files(&source);
}

#[test]
fn spec_6_1_run_honors_sample_rate_and_block_size_without_changing_samples() {
    let source = TempSource::new("short", "let out = sine(480); render 1ms;");
    let (first, first_path) = run_output(
        &source,
        "first.wav",
        &["--sample-rate", "8000", "--block-size", "1"],
    );
    let (second, second_path) = run_output(
        &source,
        "second.wav",
        &["--sample-rate", "8000", "--block-size", "256"],
    );

    assert!(first.status.success(), "stderr: {}", stderr(&first));
    assert!(second.status.success(), "stderr: {}", stderr(&second));
    assert!(stdout(&first).contains("samples: 8"));
    assert!(stdout(&second).contains("samples: 8"));
    assert_eq!(
        fs::read(&first_path).unwrap(),
        fs::read(&second_path).unwrap()
    );
}

#[test]
fn spec_6_1_existing_output_requires_force_and_is_not_modified_on_usage_error() {
    let source = TempSource::new("beep", include_str!("../fixtures/valid/stage-01-beep.ln"));
    let wav_path = output_path(&source, "beep.wav");
    fs::write(&wav_path, b"original").expect("the existing output should be written");
    let (output, _) = run_output(&source, "beep.wav", &[]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("error[E5000]"));
    assert!(stderr(&output).contains("--force"));
    assert_eq!(fs::read(&wav_path).unwrap(), b"original");
}

#[cfg(unix)]
#[test]
fn regression_stage_02_broken_symlink_output_requires_force() {
    use std::os::unix::fs::symlink;

    let source = TempSource::new(
        "atomic-output",
        include_str!("../fixtures/regression/stage-02-atomic-output.ln"),
    );
    let wav_path = output_path(&source, "linked.wav");
    symlink("missing-target.wav", &wav_path).expect("the broken symlink should be created");
    let (output, _) = run_output(&source, "linked.wav", &[]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("error[E5000]"));
    assert!(stderr(&output).contains("--force"));
    assert!(
        fs::symlink_metadata(&wav_path)
            .expect("the original symlink should remain")
            .file_type()
            .is_symlink()
    );
    assert_no_temporary_files(&source);
}

#[test]
fn spec_6_1_force_replaces_existing_output_only_after_success() {
    let source = TempSource::new("beep", include_str!("../fixtures/valid/stage-01-beep.ln"));
    let wav_path = output_path(&source, "beep.wav");
    fs::write(&wav_path, b"original").expect("the existing output should be written");
    let (output, _) = run_output(&source, "beep.wav", &["--force"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let wav = fs::read(&wav_path).expect("the replaced WAV should be readable");
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(wav.len(), 44 + 96_000 * 2);
}

#[test]
fn spec_9_invalid_source_does_not_create_output() {
    let source = TempSource::new(
        "unknown-name",
        include_str!("../fixtures/invalid/stage-02-unknown-name.ln"),
    );
    let (output, wav_path) = run_output(&source, "invalid.wav", &[]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("error[E2001]"));
    assert!(!wav_path.exists());
    assert_no_temporary_files(&source);
}

#[test]
fn spec_6_3_runtime_error_does_not_publish_partial_wav() {
    let source = TempSource::new(
        "runtime-nonfinite",
        include_str!("../fixtures/invalid/stage-02-runtime-nonfinite.ln"),
    );
    let (output, wav_path) = run_output(&source, "runtime.wav", &[]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("error[E4001]"));
    assert!(stderr(&output).contains("sample index: 0"));
    assert!(!wav_path.exists());
    assert_no_temporary_files(&source);
}

#[test]
fn spec_6_3_runtime_error_with_force_preserves_previous_output() {
    let source = TempSource::new(
        "runtime-nonfinite",
        include_str!("../fixtures/invalid/stage-02-runtime-nonfinite.ln"),
    );
    let wav_path = output_path(&source, "runtime.wav");
    fs::write(&wav_path, b"original").expect("the existing output should be written");
    let (output, _) = run_output(&source, "runtime.wav", &["--force"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("error[E4001]"));
    assert_eq!(fs::read(&wav_path).unwrap(), b"original");
    assert_no_temporary_files(&source);
}

#[test]
fn spec_6_4_max_seconds_rejects_render_before_creating_output() {
    let source = TempSource::new("too-long", "let out = 0; render 601s;");
    let (output, wav_path) = run_output(&source, "too-long.wav", &[]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("error[E4003]"));
    assert!(!wav_path.exists());
    assert_no_temporary_files(&source);
}

#[test]
fn spec_5_11_independent_run_processes_have_identical_wav_bytes() {
    let first = TempSource::new("shared", include_str!("../fixtures/valid/stage-01-beep.ln"));
    let second = TempSource::new("shared", include_str!("../fixtures/valid/stage-01-beep.ln"));
    assert_ne!(first.path().parent(), second.path().parent());

    let (first_output, first_wav) = run_output(&first, "first.wav", &[]);
    let (second_output, second_wav) = run_output(&second, "second.wav", &[]);
    assert!(
        first_output.status.success(),
        "stderr: {}",
        stderr(&first_output)
    );
    assert!(
        second_output.status.success(),
        "stderr: {}",
        stderr(&second_output)
    );
    assert_eq!(fs::read(first_wav).unwrap(), fs::read(second_wav).unwrap());
}
