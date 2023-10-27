use std::env;
use std::error::Error;
use std::io;
use std::process;
use transmuter::{Mutation, StringResult};

mod command;
mod interactive;

#[derive(Clone, Copy)]
enum Mode{
    Interactive,
    NonInteractive(Mutation),
}

fn main() {
    let result = get_mode()
        .and_then(|mode|run(&mode));
    match result {
        Ok(result) => println!("{}", result),
        Err(error) => {
            eprintln!("\x1b[0;31m{}\x1b[0m", error);
            process::exit(1);
        },
    }
}

fn run(mode: &Mode) -> StringResult {
    match mode {
        Mode::NonInteractive(mutation) => {
            eprintln!("Will apply {}:", mutation);
            let stdin = get_stdin()?;
            mutation.mutate(stdin)
        },
        Mode::Interactive => {
            interactive::enter_loop();
            Ok("ok".into())
            }
    }
}

/// Get standard input as string or return error
pub fn get_stdin() -> StringResult {
    let mut input = String::new();
    let lines = io::stdin().lines();
    for line in lines {
        input.push_str(&line?);
        input.push('\n');
    }
    Ok(input)
}

fn get_mode() -> Result<Mode, Box<dyn Error>> {
    if env::args().len() == 1{
        Ok(Mode::Interactive)
    } else {
        let mutation = env::args().nth(1).ok_or("Error reading mutation")?;
        Ok(Mode::NonInteractive(Mutation::try_from(&mutation as &str)?))
    }
}
