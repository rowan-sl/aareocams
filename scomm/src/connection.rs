use bytes::BytesMut;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::VecDeque,
    fmt::{self, Debug, Formatter},
    io,
    marker::PhantomData,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

use crate::header::Header;

#[derive(thiserror::Error, Debug)]
pub enum ConnectionRecvError {
    #[error("Disconnected!")]
    Disconnected,
    #[error("Failed to decode incoming message:\n{0}")]
    DecodeError(#[from] DecodeError),
    #[error("IO Error while reading from socket!\n{0}")]
    IOError(#[from] io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ConnectionSendError {
    #[error("Disconnected!")]
    Disconnected,
    #[error("IO Error while writing to socket!\n{0}")]
    IOError(#[from] io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("Failed to decode message:\n{0}")]
    DeserializeMessage(#[from] bincode::Error),
    #[error("Failed to decode header!\n{0}")]
    DeserializeHeader(#[from] crate::header::DecodeHeaderError),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Connection<
    M: Serialize + DeserializeOwned, /* message type to be used with the connection, held as a static type for usage seminatics */
    O: bincode::Options + Clone,
> {
    stream: TcpStream,
    pending_send_data: VecDeque<BytesMut>,
    pending_recv_data: Vec<u8>,
    pending_received: VecDeque<M>,
    /// if `None`, this means it is waiting for a header, if `Some` that means
    /// data is currently being read into the receiving buffer
    current_message_header: Option<Header<M, O>>,
    #[derivative(Debug = "ignore")]
    seri_opts: O,
    _msg_type: PhantomData<M>,
}

impl<
        M: Serialize + DeserializeOwned, /* message type to be used with the connection, held as a static type for usage seminatics */
        O: bincode::Options + Clone,
    > Connection<M, O>
{
    /// creates a new [`Connection`] from a tcp stream
    pub fn new(stream: TcpStream, seri_opts: O) -> Self {
        Self {
            stream,
            seri_opts,

            pending_received: VecDeque::new(),
            pending_recv_data: vec![],
            pending_send_data: VecDeque::new(),
            current_message_header: None,
            _msg_type: PhantomData,
        }
    }

    /// shuts down the write half of the socket, meaning all future writes will fail
    ///
    /// does NOT write any remaining data, so use send_all before calling this
    pub async fn shutdown(&mut self) -> io::Result<()> {
        self.stream.shutdown().await?;
        Ok(())
    }

    /// Queue a new message to be sent
    pub fn queue(&mut self, message: &M) -> Result<(), bincode::Error> {
        let header = Header::header_for(message, self.seri_opts.clone())?;
        let message = self.seri_opts.clone().serialize(message)?;
        self.pending_send_data
            .push_front(header.serialize_header().into_iter().collect());
        self.pending_send_data
            .push_front(message.into_iter().collect());
        Ok(())
    }

    /// Send some data, if there is any to send
    pub async fn send(&mut self) -> Result<(), ConnectionSendError> {
        if let Some(buf) = self.pending_send_data.back_mut() {
            if self.stream.write_buf(buf).await? == 0 {
                if buf.is_empty() {
                    drop(self.pending_send_data.pop_back());
                } else {
                    return Err(ConnectionSendError::Disconnected);
                }
            }
        }
        Ok(())
    }

    /// send all queued data
    pub async fn send_all(&mut self) -> Result<(), ConnectionSendError> {
        while !self.pending_send_data.is_empty() {
            self.send().await?
        }
        Ok(())
    }

    pub async fn wait_for_readable(&mut self) -> io::Result<()> {
        self.stream.readable().await
    }

    /// # Returns
    /// if a new message has been decoded
    ///
    /// if decoding fails, the state of the stream and buffers may be compromised, and it would be a good idea to reset the stream
    ///
    /// TODO implement a method of traversing along the buffer, attempting to decode a header, in order to fix the aformentioned issue
    pub fn attempt_decode(&mut self) -> Result<bool, DecodeError> {
        match self.current_message_header.clone() {
            None => {
                if self.pending_recv_data.len() >= Header::<M, O>::header_byte_size() {
                    self.current_message_header = Some(Header::<M, O>::decode_from_bytes(
                        &self
                            .pending_recv_data
                            .drain(..Header::<M, O>::header_byte_size())
                            .collect::<Vec<u8>>(),
                        self.seri_opts.clone(),
                    )?);
                }
                Ok(false)
            }
            Some(header) => {
                if header.size() <= self.pending_recv_data.len() as u64 {
                    let data = self
                        .pending_recv_data
                        .drain(..header.size() as usize)
                        .collect::<Vec<u8>>();
                    let message = self.seri_opts.clone().deserialize(&data)?;
                    self.pending_received.push_front(message);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// reads some data from the socket, and attempts to decode a message if enough data is sent
    ///
    /// # Returns
    /// if a new message has been read, or if there was a error while reading
    pub async fn recv(&mut self) -> Result<bool, ConnectionRecvError> {
        self.stream.read_buf(&mut self.pending_recv_data).await?;
        let mut message_decoded = false;
        while self.attempt_decode()? {
            message_decoded = true;
        }
        Ok(message_decoded)
    }

    /// continuously receives data untill at least one new message comes through
    pub async fn recv_msg(&mut self) -> Result<(), ConnectionRecvError> {
        while !self.recv().await? {}
        Ok(())
    }

    /// get the least recent message received, if one exists
    ///
    /// messages are received in the order that they are sent in
    pub fn get(&mut self) -> Option<M> {
        self.pending_received.pop_back()
    }

    pub fn into_split(self) -> (ConnectionReadHalf<M, O>, ConnectionWriteHalf<M, O>) {
        let (read_stream, write_stream) = self.stream.into_split();
        (
            ConnectionReadHalf {
                read_stream,
                pending_recv_data: self.pending_recv_data,
                pending_received: self.pending_received,
                current_message_header: self.current_message_header,
                seri_opts: self.seri_opts.clone(),
                _msg_type: PhantomData,
            },
            ConnectionWriteHalf {
                write_stream,
                pending_send_data: self.pending_send_data,
                seri_opts: self.seri_opts.clone(),
                _msg_type: PhantomData,
            },
        )
    }
}

// impl<
//     M: Serialize + DeserializeOwned + Debug, /* message type to be used with the connection, held as a static type for usage seminatics */
//     O: bincode::Options + Clone,
// > Debug for Connection<M, O> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.debug_struct("Connection")
//             .field("stream", &self.stream)
//             .field("pending_send_data", &self.pending_send_data)
//             .field("pending_recv_data", &self.pending_recv_data)
//             .field("pending_received", &self.pending_received)
//             .field("current_message_header", &"Header")
//             .field("seri_opts", &"Options")
//             .field("_msg_type", &self._msg_type)
//             .finish()
//     }
// }

#[derive(Debug)]
pub struct ConnectionReadHalf<
    M: Serialize + DeserializeOwned, /* message type to be used with the connection, held as a static type for usage seminatics */
    O: bincode::Options + Clone,
> {
    read_stream: OwnedReadHalf,
    pending_recv_data: Vec<u8>,
    pending_received: VecDeque<M>,
    /// if `None`, this means it is waiting for a header, if `Some` that means
    /// data is currently being read into the receiving buffer
    current_message_header: Option<Header<M, O>>,
    seri_opts: O,
    _msg_type: PhantomData<M>,
}

impl<
        M: Serialize + DeserializeOwned, /* message type to be used with the connection, held as a static type for usage seminatics */
        O: bincode::Options + Clone,
    > ConnectionReadHalf<M, O>
{
    pub async fn wait_for_readable(&mut self) -> io::Result<()> {
        self.read_stream.readable().await
    }

    /// # Returns
    /// if a new message has been decoded
    ///
    /// if decoding fails, the state of the stream and buffers may be compromised, and it would be a good idea to reset the stream
    ///
    /// TODO implement a method of traversing along the buffer, attempting to decode a header, in order to fix the aformentioned issue
    pub fn attempt_decode(&mut self) -> Result<bool, DecodeError> {
        match self.current_message_header.clone() {
            None => {
                if self.pending_recv_data.len() >= Header::<M, O>::header_byte_size() {
                    self.current_message_header = Some(Header::<M, O>::decode_from_bytes(
                        &self
                            .pending_recv_data
                            .drain(..Header::<M, O>::header_byte_size())
                            .collect::<Vec<u8>>(),
                        self.seri_opts.clone(),
                    )?);
                }
                Ok(false)
            }
            Some(header) => {
                if header.size() <= self.pending_recv_data.len() as u64 {
                    let data = self
                        .pending_recv_data
                        .drain(..header.size() as usize)
                        .collect::<Vec<u8>>();
                    let message = self.seri_opts.clone().deserialize(&data)?;
                    self.pending_received.push_front(message);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// reads some data from the socket, and attempts to decode a message if enough data is sent
    ///
    /// # Cancel Saftey
    /// this IS cancelation safe
    ///
    /// # Returns
    /// if a new message has been read, or if there was a error while reading
    pub async fn recv(&mut self) -> Result<bool, ConnectionRecvError> {
        self.read_stream
            .read_buf(&mut self.pending_recv_data)
            .await?;
        let mut message_decoded = false;
        while self.attempt_decode()? {
            message_decoded = true;
        }
        Ok(message_decoded)
    }

    /// continuously receives data untill at least one new message comes through
    pub async fn recv_msg(&mut self) -> Result<(), ConnectionRecvError> {
        while !self.recv().await? {}
        Ok(())
    }

    /// get the least recent message received, if one exists
    ///
    /// messages are received in the order that they are sent in
    pub fn get(&mut self) -> Option<M> {
        self.pending_received.pop_back()
    }
}

pub struct ConnectionWriteHalf<
    M: Serialize + DeserializeOwned, /* message type to be used with the connection, held as a static type for usage seminatics */
    O: bincode::Options + Clone,
> {
    write_stream: OwnedWriteHalf,
    pending_send_data: VecDeque<BytesMut>,
    seri_opts: O,
    _msg_type: PhantomData<M>,
}

impl<
        M: Serialize + DeserializeOwned, /* message type to be used with the connection, held as a static type for usage seminatics */
        O: bincode::Options + Clone,
    > ConnectionWriteHalf<M, O>
{
    /// shuts down the write half of the socket, meaning all future writes will fail
    ///
    /// does NOT write any remaining data, so use send_all before calling this
    pub async fn shutdown(&mut self) -> io::Result<()> {
        self.write_stream.shutdown().await?;
        Ok(())
    }

    /// Queue a new message to be sent
    pub fn queue(&mut self, message: &M) -> Result<(), bincode::Error> {
        let header = Header::header_for(message, self.seri_opts.clone())?;
        let message = self.seri_opts.clone().serialize(message)?;
        self.pending_send_data
            .push_front(header.serialize_header().into_iter().collect());
        self.pending_send_data
            .push_front(message.into_iter().collect());
        Ok(())
    }

    /// Send some data, if there is any to send
    pub async fn send(&mut self) -> Result<(), ConnectionSendError> {
        if let Some(buf) = self.pending_send_data.back_mut() {
            if self.write_stream.write_buf(buf).await? == 0 {
                if buf.is_empty() {
                    drop(self.pending_send_data.pop_back());
                } else {
                    return Err(ConnectionSendError::Disconnected);
                }
            }
        }
        Ok(())
    }

    /// send all queued data
    pub async fn send_all(&mut self) -> Result<(), ConnectionSendError> {
        while !self.pending_send_data.is_empty() {
            self.send().await?
        }
        Ok(())
    }
}

impl<M: Serialize + DeserializeOwned, O: Copy + bincode::Options> Debug
    for ConnectionWriteHalf<M, O>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionWriteHalf")
            .field("write_stream", &self.write_stream)
            .field("pending_send_data", &self.pending_send_data)
            .field("seri_opts", &"")
            .finish()
    }
}
