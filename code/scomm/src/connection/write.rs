use super::ser::Writer;
pub use super::ser::WriterSinkErr;
use bincode::Options as BincodeOptions;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{self, AsyncWriteExt},
    net::tcp::OwnedWriteHalf,
};

#[derive(Debug, thiserror::Error)]
#[error("Failed to queue a message:\n{0}")]
pub struct QueueError(#[from] WriterSinkErr);

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Disconnected while writing to socket!")]
    Disconnected,
    #[error("Failed to write to socket!\n{0}")]
    WriteErr(io::Error),
}

#[derive(Debug)]
pub struct SocketWriter<M: Serialize + DeserializeOwned, O: BincodeOptions + Clone> {
    socket: OwnedWriteHalf,
    writer: Writer<M, O>,
}

impl<M: Serialize + DeserializeOwned, O: BincodeOptions + Clone> SocketWriter<M, O> {
    pub fn new(writer: OwnedWriteHalf, opts: O) -> Self {
        Self {
            socket: writer,
            writer: Writer::new(opts),
        }
    }

    pub fn into_writer(self) -> OwnedWriteHalf {
        self.socket
    }

    pub fn queue(&mut self, msg: &M) -> Result<(), QueueError> {
        self.writer.sink(msg)?;
        Ok(())
    }

    /// Writes all of the buffered data into the socket
    ///
    /// returns if writing is done
    ///
    /// # Cancel saftey
    /// this method IS cancel safe, if used in a select! statement, it is guareteed that the writing will continue sucessfully the next time this method is called
    pub async fn update(&mut self) -> Result<bool, UpdateError> {
        match self.socket.write_buf(self.writer.as_byte_source()).await {
            Ok(0) => {
                if self.writer.buf_len() != 0 {
                    return Err(UpdateError::Disconnected);
                } else {
                    Ok(true)
                }
            }
            Ok(_) => Ok(false),
            Err(e) => {
                return Err(UpdateError::WriteErr(e));
            }
        }
    }
}
