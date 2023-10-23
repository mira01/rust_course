use std::env;
use std::error::Error;
use std::process;
use transmuter::{Mutation, StringResult};

fn main() {
    match run() {
        Ok(output) => print!("{}", output),
        Err(error) => {
            eprintln!("\x1b[0;31m{}\x1b[0m", error);
            process::exit(1);
        }
    };
}

fn run() -> StringResult {
    let mutation = get_mutation()?;
    eprintln!("Will apply {}:", mutation);
    mutation.mutate()
}

fn get_mutation() -> Result<Mutation, Box<dyn Error>> {
    let mutation = env::args().nth(1).ok_or("Cli argument not provided")?;
    Mutation::try_from(&mutation as &str)
}
