use std::env;
use std::ffi::OsStr;
use std::process::ExitCode;

const HELP: &str = concat!(
    "linen ",
    env!("CARGO_PKG_VERSION"),
    "\n",
    "An experimental functional language for audio signals.\n",
    "Compiler and audio rendering commands are not implemented yet.\n\n",
    "Usage:\n",
    "  linen --help\n",
    "  linen --version\n"
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
        _ => {
            eprint!("error: expected --help or --version\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}

fn is_help(argument: &OsStr) -> bool {
    argument == "-h" || argument == "--help"
}

fn is_version(argument: &OsStr) -> bool {
    argument == "-V" || argument == "--version"
}
