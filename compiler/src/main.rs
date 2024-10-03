use std::env;
use std::io::Read;

use compiler::error::LinenError;
use compiler::lexer::scanner::Scanner;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn run_file(path: String) {
    let mut file = std::fs::File::open(path.clone()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let _ = run(contents, path);
}

fn run(source: String, place: String) -> Result<(), LinenError> {
    let mut scanner = Scanner::new(source, place);
    let tokens = scanner.scan_tokens()?;

    for token in tokens {
        println!("{:?}", token);
    }
    Ok(())
}

fn run_prompt() -> Result<(), String> {
    let mut rl = DefaultEditor::new().unwrap();

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline("linen> ");

        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line.as_str());
                let _ = run(line, "<REPL>".to_string());
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
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    let _ = rl.save_history("history.txt");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        run_file(args[1].clone());
    } else {
        let _ = run_prompt();
    }
}
