//! CLI compiler and runner for the Linen programming language.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "linc")]
#[command(about = "Linen compiler and runner")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a file and print the AST
    Parse {
        /// Input file
        file: PathBuf,
    },
    /// Typecheck a file
    Check {
        /// Input file
        file: PathBuf,
    },
    /// Run a file and bounce to WAV
    Run {
        /// Input file
        file: PathBuf,
        /// Output WAV file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Duration in seconds
        #[arg(short, long)]
        duration: Option<f64>,
    },
    /// Start the REPL
    Repl,
    /// Desugar a file and print Core AST
    Desugar {
        /// Input file
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { file } => {
            let source = fs::read_to_string(&file).expect("Failed to read file");
            match linen_lexer::lex(&source) {
                Ok(tokens) => {
                    let mut parser = linen_parser::Parser::new(&tokens, &source);
                    match parser.parse_module() {
                        Ok(ast) => println!("{:#?}", ast),
                        Err(e) => eprintln!("Parse error: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Lex error: {:?}", e),
            }
        }
        Commands::Check { file } => {
            let source = fs::read_to_string(&file).expect("Failed to read file");
            match linen_lexer::lex(&source) {
                Ok(tokens) => {
                    let mut parser = linen_parser::Parser::new(&tokens, &source);
                    match parser.parse_module() {
                        Ok(_ast) => {
                            // TODO: run type checker
                            println!("Typecheck: OK (not yet implemented)");
                        }
                        Err(e) => eprintln!("Parse error: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Lex error: {:?}", e),
            }
        }
        Commands::Run {
            file,
            output,
            duration,
        } => {
            let _output_path = output.unwrap_or_else(|| {
                let mut p = file.clone();
                p.set_extension("wav");
                p
            });
            let _duration = duration.unwrap_or(5.0);
            let source = fs::read_to_string(&file).expect("Failed to read file");

            match linen_lexer::lex(&source) {
                Ok(tokens) => {
                    let mut parser = linen_parser::Parser::new(&tokens, &source);
                    match parser.parse_module() {
                        Ok(ast) => {
                            let core = linen_core::desugar::desugar_module(&ast);
                            let _env = linen_core::interpreter::eval_module(&core);
                            // TODO: extract export and bounce to WAV
                            println!("Run: executed (WAV export not yet implemented)");
                        }
                        Err(e) => eprintln!("Parse error: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Lex error: {:?}", e),
            }
        }
        Commands::Repl => {
            println!("Starting Linen REPL...");
            // TODO: delegate to linen-repl binary or implement inline
        }
        Commands::Desugar { file } => {
            let source = fs::read_to_string(&file).expect("Failed to read file");
            match linen_lexer::lex(&source) {
                Ok(tokens) => {
                    let mut parser = linen_parser::Parser::new(&tokens, &source);
                    match parser.parse_module() {
                        Ok(ast) => {
                            let core = linen_core::desugar::desugar_module(&ast);
                            println!("{:#?}", core);
                        }
                        Err(e) => eprintln!("Parse error: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Lex error: {:?}", e),
            }
        }
    }
}
