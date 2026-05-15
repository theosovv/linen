//! Interactive REPL for the Linen programming language.

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Linen REPL v0.1.0");
    println!("Type :quit to exit, :help for commands");

    let mut rl = DefaultEditor::new()?;
    let mut env: HashMap<String, linen_core::value::Value> = HashMap::new();
    let mut type_env = linen_core::types::TypeEnv::new();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line)?;

                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if trimmed.starts_with(':') {
                    match handle_command(trimmed, &mut env, &mut type_env) {
                        Ok(should_quit) => {
                            if should_quit {
                                break;
                            }
                        }
                        Err(e) => eprintln!("Error: {:?}", e),
                    }
                    continue;
                }

                // Evaluate expression
                match linen_lexer::lex(trimmed) {
                    Ok(tokens) => {
                        let mut parser = linen_parser::Parser::new(&tokens, trimmed);
                        match parser.parse_module() {
                            Ok(ast) => {
                                let core = linen_core::desugar::desugar_module(&ast);
                                let new_env = linen_core::interpreter::eval_module(&core);
                                for (name, value) in &new_env {
                                    println!("{} = {:?}", name, value);
                                }
                                env.extend(new_env);
                            }
                            Err(e) => eprintln!("Parse error: {:?}", e),
                        }
                    }
                    Err(e) => eprintln!("Lex error: {:?}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn handle_command(
    line: &str,
    _env: &mut HashMap<String, linen_core::value::Value>,
    _type_env: &mut linen_core::types::TypeEnv,
) -> std::result::Result<bool, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts.first().copied() {
        Some(":quit") => Ok(true),
        Some(":help") => {
            println!("Commands:");
            println!("  :type <expr>    - Show type of expression");
            println!("  :export <expr> <file> <dur> - Export signal to WAV");
            println!("  :load <file>    - Load module into REPL");
            println!("  :info <ident>   - Show documentation");
            println!("  :quit           - Exit REPL");
            Ok(false)
        }
        Some(":type") => {
            if parts.len() < 2 {
                println!("Usage: :type <expr>");
            } else {
                println!("Type inference not yet implemented");
            }
            Ok(false)
        }
        Some(":export") => {
            println!("Export not yet implemented");
            Ok(false)
        }
        Some(":load") => {
            println!("Load not yet implemented");
            Ok(false)
        }
        Some(":info") => {
            println!("Info not yet implemented");
            Ok(false)
        }
        _ => {
            println!("Unknown command. Type :help for available commands.");
            Ok(false)
        }
    }
}
