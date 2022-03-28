use super::ser::Reader;
pub use super::ser::UpdateReaderError;
use bincode::Options as BincodeOptions;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{self, AsyncReadExt},
    net::tcp::OwnedReadHalf,
};

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Disconnected while writing to socket!")]
    Disconnected,
    #[error("Failed to read from socket!\n{0}")]
    Read(#[from] io::Error),
    #[error("Failed to update reader!\n{0}")]
    ReaderUpdate(#[from] UpdateReaderError),
}

#[derive(Debug)]
pub struct SocketReader<M: Serialize + DeserializeOwned, O: BincodeOptions + Clone> {
    socket: OwnedReadHalf,
    reader: Reader<M, O>,
}

impl<M: Serialize + DeserializeOwned, O: BincodeOptions + Clone> SocketReader<M, O> {
    pub fn new(reader: OwnedReadHalf, opts: O) -> Self {
        Self {
            socket: reader,
            reader: Reader::new(opts),
        }
    }

    pub fn into_reader(self) -> OwnedReadHalf {
        self.socket
    }

    pub fn get_next(&mut self) -> Option<M> {
        self.reader.get_next()
    }

    /// # Cancel Saftey
    /// perfectly cancelation safe
    pub async fn update(&mut self) -> Result<bool, UpdateError> {
        let mut new = false;
        //clean code right here guys
        while self.reader.full_update()? {
            new = true;
        }
        // looks bad, but there is a reason
        if new {
            Ok(true)
        } else {
            let read = self.socket.read_buf(self.reader.as_byte_sink()).await?;
            //buffer remaining can never be zero, so if the return is zero there must be an error
            if 0 == read {
                return Err(UpdateError::Disconnected);
            }
            Ok(false)
        }
    }
}
