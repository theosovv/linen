//! Linen CLI
//!
//! Command-line interface for the Linen programming language:
//! - Compile .ln files
//! - Run programs
//! - Interactive REPL
//! - Package management

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Parser)]
#[command(name = "linen")]
#[command(about = "Linen - Functional-Reactive Music Programming Language")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Linen program
    Run {
        /// Path to source file
        file: String,
        /// Enable JIT compilation
        #[arg(long)]
        jit: bool,
    },
    /// Compile to bytecode
    Compile {
        /// Path to source file
        file: String,
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Start interactive REPL
    Repl,
    /// Check file for errors without running
    Check {
        /// Path to source file
        file: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, jit } => {
            info!("Running {} (JIT: {})", file, jit);
            println!(
                "Linen {} - Run mode not yet implemented",
                linen_compiler::VERSION
            );
            todo!("Implement run command")
        }
        Commands::Compile { file, output } => {
            info!("Compiling {} to {:?}", file, output);
            println!(
                "Linen {} - Compile mode not yet implemented",
                linen_compiler::VERSION
            );
            todo!("Implement compile command")
        }
        Commands::Repl => {
            info!("Starting REPL");
            println!(
                "Linen {} REPL - not yet implemented",
                linen_compiler::VERSION
            );
            todo!("Implement REPL")
        }
        Commands::Check { file } => {
            info!("Checking {}", file);
            println!(
                "Linen {} - Check mode not yet implemented",
                linen_compiler::VERSION
            );
            todo!("Implement check command")
        }
    }
}
