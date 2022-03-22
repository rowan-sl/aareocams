use aareocams_scomm::{
    connection::{ConnectionRecvError, ConnectionSendError},
    Connection,
};
use iced_native::subscription::{self, Subscription};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use tokio::{io, net::ToSocketAddrs, select};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};

#[derive(thiserror::Error, Debug)]
pub enum Error<A: ToSocketAddrs> {
    #[error("Failed to receive message {0}")]
    RecvErr(#[from] ConnectionRecvError),
    #[error("Failed to send message {0}")]
    SendErr(#[from] ConnectionSendError),
    #[error("Failed to serialize message {0}")]
    SeriErr(#[from] bincode::Error),
    #[error("Failed to connect to {0}:\n{1}")]
    ConnectionErr(A, io::Error),
    #[error("Message sender stream closed before a close event was received!")]
    MessageChannelClosed,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum Event<A: ToSocketAddrs + Debug, M: Serialize + DeserializeOwned + Debug> {
    Error(Error<A>),
    Init {
        /// closes the channel, sending all pending data and returning the Connection
        close_sig_send: oneshot::Sender<()>,
        msg_send: mpsc::UnboundedSender<M>,
    },
    Received(M),
    Closed {
        #[derivative(Debug = "ignore")]
        stream: Connection<M, bincode::DefaultOptions>,
    },
}

enum State<A: ToSocketAddrs, M: Serialize + DeserializeOwned> {
    Uninitialized {
        ip: A,
    },
    Running {
        stream: Connection<M, bincode::DefaultOptions>,
        close_sig_recv: oneshot::Receiver<()>,
        msg_recv: mpsc::UnboundedReceiver<M>,
    },
    Closed,
}

pub fn like_and_subscribe<
    A: ToSocketAddrs + Clone + Sync + Debug + Send + 'static,
    M: Serialize + DeserializeOwned + Debug + Send + 'static,
>(
    ip: A,
) -> Subscription<Event<A, M>> {
    struct ID;

    subscription::unfold(
        std::any::TypeId::of::<ID>(),
        State::Uninitialized { ip },
        move |mut state: State<A, M>| async move {
            match state {
                State::Uninitialized { ip } => {
                    let stream = match TcpStream::connect(ip.clone()).await {
                        Ok(s) => s,
                        Err(e) => {
                            return (
                                Some(Event::Error(Error::ConnectionErr(ip, e))),
                                State::Closed,
                            );
                        }
                    };
                    let connection = Connection::<M, bincode::DefaultOptions>::new(
                        stream,
                        bincode::DefaultOptions::new(),
                    );
                    let (close_tx, close_rx) = oneshot::channel();
                    let (msg_tx, msg_rx) = mpsc::unbounded_channel();
                    (
                        Some(Event::Init {
                            close_sig_send: close_tx,
                            msg_send: msg_tx,
                        }),
                        State::Running {
                            stream: connection,
                            close_sig_recv: close_rx,
                            msg_recv: msg_rx,
                        },
                    )
                }
                State::Running {
                    ref mut stream,
                    ref mut close_sig_recv,
                    ref mut msg_recv,
                } => {
                    if let Some(message) = stream.get() {
                        return (Some(Event::Received(message)), state);
                    }

                    if let Err(e) = stream.send().await {
                        return (Some(Event::Error(e.into())), State::Closed);
                    }

                    select!(
                        to_send = msg_recv.recv() => {
                            println!("Sending message\n{:#?}", to_send);
                            if let Some(msg) = to_send {
                                if let Err(e) = stream.queue(&msg) {
                                    return (
                                        Some(Event::Error(e.into())),
                                        State::Closed
                                    )
                                }
                            } else {
                                return (
                                    Some(Event::Error(Error::MessageChannelClosed)),
                                    State::Closed
                                )
                            }
                        }
                        _ = close_sig_recv => {
                            return (
                                Some(Event::Closed {
                                    stream: match state {
                                        State::Running {stream, ..} => stream,
                                        _ => unreachable!()
                                    }
                                }),
                                State::Closed,
                            )
                        }
                        res = stream.recv() => {
                            match res {
                                Ok(message_received) => {
                                    if message_received {
                                        if let Some(message) = stream.get() {
                                            return (Some(Event::Received(message)), state)
                                        }
                                    }
                                }
                                Err(e) => {
                                    return (Some(Event::Error(e.into())), State::Closed)
                                }
                            }
                        }
                    );
                    (None, state)
                }
                State::Closed => (None, State::Closed),
            }
        },
    )
}
