use serde::{Serialize, Deserialize};
use serde_json;
use std::error::Error;
use std::io::Write;

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
}

impl TryFrom<&[u8]> for Message {
    type Error = Box<dyn Error>;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        Ok(serde_json::from_slice(&data)?)
    }
}
