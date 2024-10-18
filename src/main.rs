use linen::exec::Exec;

fn main() -> Result<(), String> {
    let mut exec = Exec::new();

    exec.run(None);

    Ok(())
}
