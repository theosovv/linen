use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use linen::{
    CheckOutcome, CompileConfig, CompileOutcome, Diagnostic, RenderConfig, RenderConfigError,
    SourceMap,
};

const HELP: &str = concat!(
    "linen ",
    env!("CARGO_PKG_VERSION"),
    "\n",
    "An experimental functional language for audio signals.\n\n",
    "Usage:\n",
    "  linen --help\n",
    "  linen --version\n",
    "  linen check <input.ln>\n",
    "  linen run <input.ln> --output <output.wav> [--sample-rate <hz>] [--block-size <n>] [--max-seconds <n>] [--force]\n"
);

fn main() -> ExitCode {
    let arguments: Vec<_> = env::args_os().skip(1).collect();

    match arguments.as_slice() {
        [] => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        [argument] if is_help(argument) => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        [argument] if is_version(argument) => {
            println!("linen {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [command, path] if command == "check" => run_check(Path::new(path)),
        [command, arguments @ ..] if command == "run" => run_from_args(arguments),
        _ => {
            eprint!(
                "error: expected --help, --version, check <input.ln>, or run <input.ln> --output <output.wav>\n\n{HELP}"
            );
            ExitCode::from(2)
        }
    }
}

fn run_check(path: &Path) -> ExitCode {
    let (logical_name, source) = match read_input(path) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    let CheckOutcome { sources, result } =
        linen::check_with_config(&logical_name, &source, CompileConfig::default());

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(diagnostic) => source_error(&sources, &diagnostic),
    }
}

fn run_from_args(arguments: &[OsString]) -> ExitCode {
    let options = match RunOptions::parse(arguments) {
        Ok(options) => options,
        Err(ArgumentError::Usage(message)) => {
            eprint!("error: {message}\n\n{HELP}");
            return ExitCode::from(2);
        }
        Err(ArgumentError::InvalidNumber { option, value }) => {
            eprintln!("error[E4000]: invalid numeric value '{value}' for {option}");
            return ExitCode::from(2);
        }
        Err(ArgumentError::Config(error)) => {
            eprintln!("error[E4000]: {error}");
            return ExitCode::from(2);
        }
    };

    run_render(options)
}

fn run_render(options: RunOptions) -> ExitCode {
    let (logical_name, source) = match read_input(&options.input) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };

    match fs::symlink_metadata(&options.output) {
        Ok(_) if !options.force => {
            eprintln!(
                "error[E5000]: output '{}' already exists; use --force to overwrite it",
                options.output.display()
            );
            return ExitCode::from(2);
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            eprintln!(
                "error[E5000]: could not inspect output '{}': {error}",
                options.output.display()
            );
            return ExitCode::from(2);
        }
    }

    let CompileOutcome {
        sources, result, ..
    } = match linen::compile_with_config(
        &logical_name,
        &source,
        CompileConfig::default(),
        options.config,
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            eprintln!("error[E4000]: {error}");
            return ExitCode::from(2);
        }
    };
    let program = match result {
        Ok(program) => program,
        Err(diagnostic) => return source_error(&sources, &diagnostic),
    };

    let (temporary_path, mut temporary_file) = match create_temporary_output(&options.output) {
        Ok(output) => output,
        Err(error) => {
            eprintln!(
                "error[E5000]: could not create temporary output for '{}': {error}",
                options.output.display()
            );
            return ExitCode::from(2);
        }
    };

    let result = linen::render(&program, &mut temporary_file);
    let flush_result = if result.is_ok() {
        temporary_file.flush()
    } else {
        Ok(())
    };
    drop(temporary_file);

    let report = match (result, flush_result) {
        (Ok(report), Ok(())) => report,
        (Ok(_), Err(error)) | (Err(linen::RenderError::Io(error)), _) => {
            let _ = fs::remove_file(&temporary_path);
            eprintln!(
                "error[E5000]: could not write output '{}': {error}",
                options.output.display()
            );
            return ExitCode::from(2);
        }
        (Err(linen::RenderError::Diagnostic(diagnostic)), _) => {
            let _ = fs::remove_file(&temporary_path);
            return source_error(&sources, &diagnostic);
        }
    };

    if let Err(error) = publish_output(&temporary_path, &options.output, options.force) {
        let _ = fs::remove_file(&temporary_path);
        if error.kind() == io::ErrorKind::AlreadyExists && !options.force {
            eprintln!(
                "error[E5000]: output '{}' already exists; use --force to overwrite it",
                options.output.display()
            );
        } else {
            eprintln!(
                "error[E5000]: could not publish output '{}': {error}",
                options.output.display()
            );
        }
        return ExitCode::from(2);
    }

    println!("output: {}", options.output.display());
    println!("samples: {}", report.samples);
    println!("sample rate: {} Hz", report.sample_rate);
    println!("duration: {} s", report.duration_seconds);
    ExitCode::SUCCESS
}

fn publish_output(temporary: &Path, output: &Path, force: bool) -> io::Result<()> {
    if force {
        return fs::rename(temporary, output);
    }

    fs::hard_link(temporary, output)?;
    fs::remove_file(temporary)
}

fn read_input(path: &Path) -> Result<(String, String), String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("error[E5000]: could not read '{}': {error}", path.display()))?;
    let file_name = path.file_name().ok_or_else(|| {
        format!(
            "error[E5000]: could not determine logical source name for '{}'",
            path.display()
        )
    })?;
    Ok((file_name.to_string_lossy().into_owned(), source))
}

fn source_error(sources: &SourceMap, diagnostic: &Diagnostic) -> ExitCode {
    match linen::render_diagnostic(sources, diagnostic) {
        Ok(rendered) => {
            eprint!("{rendered}");
            ExitCode::from(1)
        }
        Err(error) => {
            eprintln!("internal error: could not render source diagnostic: {error}");
            ExitCode::from(3)
        }
    }
}

fn create_temporary_output(output: &Path) -> io::Result<(PathBuf, File)> {
    let parent = output.parent().unwrap_or_else(|| Path::new(""));
    let file_name = output
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing output file name"))?
        .to_string_lossy();

    for counter in 0..1_024_u32 {
        let path = parent.join(format!(".{file_name}.{}.{counter}.tmp", std::process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate a unique temporary output name",
    ))
}

struct RunOptions {
    input: PathBuf,
    output: PathBuf,
    config: RenderConfig,
    force: bool,
}

impl RunOptions {
    fn parse(arguments: &[OsString]) -> Result<Self, ArgumentError> {
        let mut input = None;
        let mut output = None;
        let mut sample_rate = None;
        let mut block_size = None;
        let mut max_seconds = None;
        let mut force = false;
        let mut index = 0;

        while index < arguments.len() {
            let argument = &arguments[index];
            if argument == "--force" {
                if force {
                    return Err(ArgumentError::Usage("duplicate --force".into()));
                }
                force = true;
                index += 1;
                continue;
            }

            if argument == "--output"
                || argument == "--sample-rate"
                || argument == "--block-size"
                || argument == "--max-seconds"
            {
                index += 1;
                let value = arguments.get(index).ok_or_else(|| {
                    ArgumentError::Usage(format!(
                        "missing value for {}",
                        argument.to_string_lossy()
                    ))
                })?;

                if argument == "--output" {
                    if output.is_some() {
                        return Err(ArgumentError::Usage("duplicate --output".into()));
                    }
                    output = Some(PathBuf::from(value));
                } else if argument == "--sample-rate" {
                    if sample_rate.is_some() {
                        return Err(ArgumentError::Usage("duplicate --sample-rate".into()));
                    }
                    sample_rate = Some(parse_number(value, "--sample-rate")?);
                } else if argument == "--block-size" {
                    if block_size.is_some() {
                        return Err(ArgumentError::Usage("duplicate --block-size".into()));
                    }
                    block_size = Some(parse_number(value, "--block-size")?);
                } else {
                    if max_seconds.is_some() {
                        return Err(ArgumentError::Usage("duplicate --max-seconds".into()));
                    }
                    max_seconds = Some(parse_number(value, "--max-seconds")?);
                }
            } else if argument.to_string_lossy().starts_with('-') {
                return Err(ArgumentError::Usage(format!(
                    "unknown option '{}'",
                    argument.to_string_lossy()
                )));
            } else if input.is_some() {
                return Err(ArgumentError::Usage(
                    "run accepts exactly one input file".into(),
                ));
            } else {
                input = Some(PathBuf::from(argument));
            }

            index += 1;
        }

        let input = input.ok_or_else(|| ArgumentError::Usage("missing input file".into()))?;
        let output = output.ok_or_else(|| ArgumentError::Usage("missing --output".into()))?;
        let config = RenderConfig {
            sample_rate: sample_rate.unwrap_or(48_000),
            block_size: block_size.unwrap_or(128),
            max_output_seconds: max_seconds.unwrap_or(600),
        };
        config.validate().map_err(ArgumentError::Config)?;

        Ok(Self {
            input,
            output,
            config,
            force,
        })
    }
}

fn parse_number<T: std::str::FromStr>(
    value: &OsStr,
    option: &'static str,
) -> Result<T, ArgumentError> {
    value
        .to_str()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| ArgumentError::InvalidNumber {
            option,
            value: value.to_string_lossy().into_owned(),
        })
}

enum ArgumentError {
    Usage(String),
    InvalidNumber { option: &'static str, value: String },
    Config(RenderConfigError),
}

fn is_help(argument: &OsStr) -> bool {
    argument == "-h" || argument == "--help"
}

fn is_version(argument: &OsStr) -> bool {
    argument == "-V" || argument == "--version"
}
