use serde::{Serialize, Deserialize};
use serde_json;
use std::error::Error;
use std::io::{Write, Read};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Text(String),
    File(String, Vec<u8>),
    Image(Vec<u8>),
}

impl Message {

    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>> {
       Ok(serde_json::to_string(&self)?.as_bytes().to_vec()) 
    }

    pub fn write_to_stream<T: Write>(&self, stream: &mut T) -> Result<(), Box<dyn Error>> {
        let serialized = self.serialize()?;
        println!("sending {:?}", serialized);
        let len = serialized.len() as u32;
        stream.write(&len.to_be_bytes())?;
        Ok(stream.write_all(&serialized)?)
    } 

    pub fn read_from_stream<T: Read>(stream: &mut T) -> Result<Message, Box<dyn Error>> {
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes)?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut buffer = vec![0u8; len];
        stream.read_exact(&mut buffer)?;

        Message::try_from(buffer.as_slice())
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = Box<dyn Error>;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        Ok(serde_json::from_slice(&data)?)
    }
}
