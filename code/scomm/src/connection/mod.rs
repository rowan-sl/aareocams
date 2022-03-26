pub mod ser;
pub mod read;
pub mod write;

pub use read::SocketReader;
pub use write::{SocketWriter, QueueError};

use serde::{Serialize, de::DeserializeOwned};
use bincode::Options as BincodeOptions;
use tokio::{net::TcpStream, select};


#[derive(Debug, thiserror::Error)]
pub enum StreamUpdateErr {
    #[error("Failed to update reader:\n{0}")]
    Reader(#[from] read::UpdateError),
    #[error("Failed to update writer:\n{0}")]
    Writer(#[from] write::UpdateError),
}

#[derive(Debug)]
pub struct Stream<M: Serialize + DeserializeOwned, O: BincodeOptions + Clone> {
    reader: SocketReader<M, O>,
    writer: SocketWriter<M, O>,
}

impl<M: Serialize + DeserializeOwned, O: BincodeOptions + Clone> Stream<M, O> {
    pub fn new(socket: TcpStream, opts: O) -> Self {
        let (reader, writer) = socket.into_split();
        Self {
            reader: SocketReader::new(reader, opts.clone()),
            writer: SocketWriter::new(writer, opts),
        }
    }

    pub async fn update_loop(&mut self) -> Result<bool, StreamUpdateErr> {
        loop {
            select! {
                message_read = self.reader.update() => {
                    if message_read? {
                        return Ok(true);
                    }
                }
                w_done = self.writer.update() => {
                    if w_done? {
                        break;
                    }
                }
            }
        }
        loop {
            if self.reader.update().await? {
                return Ok(true);
            } 
        }
    }

    pub fn queue(&mut self, msg: &M) -> Result<(), QueueError> {
        self.writer.queue(msg)
    }

    pub fn get(&mut self) -> Option<M> {
        self.reader.get_next()
    }
}
