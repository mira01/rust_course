use crate::mutation::{Mutation, StringResult};

use crate::message::Message;
use std::error::Error;
use std::path::Path;

#[derive(Debug)]
pub enum Command {
    File(Box<Path>),
    Image(Box<Path>),
    Text(String),
    Quit,
}

impl TryFrom<&str> for Command {
    type Error = Box<dyn Error>;

    fn try_from(item: &str) -> Result<Self, Self::Error> {
        if item.starts_with(".quit"){
            Ok(Command::Quit)
        } else if item.starts_with(".file") {
            todo!()
            
        }
        else {
            Ok(Command::Text(item.to_string()))
        }
    }
}

impl TryInto<Message> for Command {
    type Error = Box<dyn Error>;
    
    fn try_into(self) -> Result<Message, Self::Error> {
        match self {
            Command::Text(text) => Ok(Message::Text(text)),
            _ => todo!()
        }
    }
}
