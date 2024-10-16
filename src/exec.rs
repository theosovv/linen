use rustyline::{error::ReadlineError, DefaultEditor};

use crate::vm::{InterpretResult, VM};

pub struct Exec {
    vm: VM,
}

impl Default for Exec {
    fn default() -> Self {
        Exec::new()
    }
}

impl Exec {
    pub fn new() -> Self {
        Exec { vm: VM::new() }
    }

    pub fn run(&mut self, file_name: Option<&str>) {
        self.vm.init_vm();

        if let Some(name) = file_name {
            self.run_file(name);
        } else {
            self.repl();
        }
    }

    fn repl(&mut self) {
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
                    self.vm.interpret(line);
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

        self.vm.free_vm();

        let _ = rl.save_history("history.txt");
    }

    fn run_file(&mut self, file_name: &str) {
        let contents = std::fs::read_to_string(file_name).expect("Could not read file");
        let result = self.vm.interpret(contents);

        if result == InterpretResult::CompileError {
            std::process::exit(65);
        }

        if result == InterpretResult::RuntimeError {
            std::process::exit(70);
        }

        self.vm.free_vm();
    }
}
