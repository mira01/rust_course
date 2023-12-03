use serde::{Deserialize, Serialize};
use bincode;
use std::io::{Read, Write};

use thiserror::Error;

/// Supported messages in chat application
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Text(String),
    File(String, Vec<u8>),
    Image(Vec<u8>),
}

#[derive(Error, Debug)]
pub enum ChatLibError {
    #[error("Cannot compose the message")]
    ComposeError,
    #[error("Cannot send the message")]
    SendError,
    #[error("Cannot receive the message")]
    ReadError,
}

impl Message {

    /// Method for message serialization. If one wants to write the message via I/O,
    /// have a look at [write_to_stream] method.
    pub fn serialize(&self) -> Result<Vec<u8>, ChatLibError> {
        Ok(bincode::serialize(&self)
           .map_err(|_e| ChatLibError::ComposeError)?
           .to_vec()
          )
    }

    /// Method for writing a message trough I/O (File, Network, ...). Serialize message and writes
    /// its size and message itself into *Write*abale stream given as parameter
    pub fn write_to_stream<T: Write>(&self, stream: &mut T) -> Result<(), ChatLibError> {
        let serialized = self.serialize()?;
        let len = serialized.len() as u32;
        stream.write_all(&len.to_be_bytes())
            .map_err(|_e| ChatLibError::SendError )?;
        Ok(stream.write_all(&serialized)
           .map_err(|_e| ChatLibError::SendError)?)
    }

    /// Method for obtaining a message from *Read*able (File, Nework, ...)
    pub fn read_from_stream<T: Read>(stream: &mut T) -> Result<Message, ChatLibError> {
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes)
            .map_err(|_e| ChatLibError:: ReadError )?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut buffer = vec![0u8; len];
        stream.read_exact(&mut buffer)
            .map_err(|_e| ChatLibError:: ReadError )?;

        Message::try_from(buffer.as_slice())
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = ChatLibError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // You can create intentional error by sending "cus" message
        if data == &vec![0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 99, 117, 115]{
            return Err(ChatLibError::ComposeError);
        }
        Ok(bincode::deserialize(data)
           .map_err(|_e| ChatLibError::ComposeError)?
          )
    }
}
