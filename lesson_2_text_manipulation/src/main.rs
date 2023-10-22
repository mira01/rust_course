use transmuter::{Mutation, StringResult};
use std::env;
use std::process;
use std::error::Error;

fn main() {
    match run() {
        Ok(output) => print!("{}", output),
        Err(error) => {
            eprintln!("{}", error);
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
