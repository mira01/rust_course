use crate::mutation::{Mutation, StringResult};

use std::error::Error;
use std::fs;

#[derive(Debug)]
pub struct Command {
    operation: Mutation,
    argument: String,
}

impl TryFrom<&str> for Command {
    type Error = Box<dyn Error>;

    fn try_from(item: &str) -> Result<Self, Self::Error> {
        let mut split = item.split(' ');
        let operation_name = split.next().ok_or("could not read operation")?;
        let operation = Mutation::try_from(operation_name)?;
        let mut argument: String = String::new();
        for part in split {
            argument.push_str(part);
            argument.push(' ');
        }
        Ok(Command {
            operation,
            argument: argument.trim().to_string(),
        })
    }
}

impl Command {
    pub fn execute(self) -> StringResult {
        let arg = match self {
            Command {
                operation: Mutation::Csv,
                argument,
            } => fs::read_to_string(argument)?,
            Command { argument, .. } => argument,
        };
        self.operation.mutate(arg)
    }
}
