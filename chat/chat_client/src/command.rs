use chat_lib::message::Message;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Commands that can be typed in applicaiton
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
        if item.starts_with(".quit") {
            Ok(Command::Quit)
        } else if item.starts_with(".file") {
            let path = item.strip_prefix(".file").unwrap();
            let path = Path::new(path.trim());
            Ok(Command::File(path.into()))
        } else if item.starts_with(".image") {
            let path = item.strip_prefix(".image").unwrap();
            let path = Path::new(path.trim());
            Ok(Command::Image(path.into()))
        } else {
            Ok(Command::Text(item.to_string()))
        }
    }
}

impl TryInto<Message> for Command {
    type Error = Box<dyn Error>;

    fn try_into(self) -> Result<Message, Self::Error> {
        match self {
            Command::Text(text) => Ok(Message::Text(text)),
            Command::File(path) => {
                let (name, content) = file_data(&path)?;
                Ok(Message::File(name, content))
            }
            Command::Image(path) => {
                let (_name, content) = file_data(&path)?;
                Ok(Message::Image(content))
            }
            Command::Quit => Err("Quit is not sendable command".into()),
        }
    }
}

/// Read data from filepath and return (file_name, content)
fn file_data(path: &Path) -> Result<(String, Vec<u8>), Box<dyn Error>> {
    let name = path
        .components()
        .last()
        .map(|component| {
            let os_str: OsString = component.as_os_str().into();
            os_str.into_string().unwrap()
        })
        .ok_or("weird path".to_string())?;
    let mut data = Vec::new();
    let file = File::open(path)?;
    BufReader::new(file).read_to_end(&mut data)?;
    Ok((name, data))
}
