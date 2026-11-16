use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use linen::{CheckOutcome, CompileConfig};

const HELP: &str = concat!(
    "linen ",
    env!("CARGO_PKG_VERSION"),
    "\n",
    "An experimental functional language for audio signals.\n",
    "Checks Beep syntax without type checking or audio rendering.\n\n",
    "Usage:\n",
    "  linen --help\n",
    "  linen --version\n",
    "  linen check <input.ln>\n"
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
        _ => {
            eprint!("error: expected --help, --version, or check <input.ln>\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}

fn run_check(path: &Path) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("error[E5000]: could not read '{}': {error}", path.display());
            return ExitCode::from(2);
        }
    };

    let Some(file_name) = path.file_name() else {
        eprintln!(
            "error[E5000]: could not determine logical source name for '{}'",
            path.display()
        );
        return ExitCode::from(2);
    };
    let logical_name = file_name.to_string_lossy();
    let CheckOutcome { sources, result } =
        linen::check_with_config(&logical_name, &source, CompileConfig::default());

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(diagnostic) => match linen::render_diagnostic(&sources, &diagnostic) {
            Ok(rendered) => {
                eprint!("{rendered}");
                ExitCode::from(1)
            }
            Err(error) => {
                eprintln!("internal error: could not render source diagnostic: {error}");
                ExitCode::from(3)
            }
        },
    }
}

fn is_help(argument: &OsStr) -> bool {
    argument == "-h" || argument == "--help"
}

fn is_version(argument: &OsStr) -> bool {
    argument == "-V" || argument == "--version"
}
