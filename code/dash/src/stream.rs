use aareocams_scomm::{Stream, connection};
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
    #[error("Failed to queue message to be sent:\n{0}")]
    Queue(#[from] connection::QueueError),
    #[error("Failed to update stream:\n{0}")]
    Update(#[from] connection::StreamUpdateErr),
    #[error("Failed to connect to {0}:\n{1}")]
    Connection(A, io::Error),
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
        stream: Stream<M, bincode::DefaultOptions>,
    },
}

enum State<A: ToSocketAddrs, M: Serialize + DeserializeOwned> {
    Uninitialized {
        ip: A,
    },
    Running {
        stream: Stream<M, bincode::DefaultOptions>,
        close_sig_recv: oneshot::Receiver<()>,
        msg_recv: mpsc::UnboundedReceiver<M>,
    },
    Closed,
}

pub fn like_and_subscribe<
    's,
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
                                Some(Event::Error(Error::Connection(ip, e))),
                                State::Closed,
                            );
                        }
                    };
                    let connection = Stream::<M, bincode::DefaultOptions>::new(
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
                    if let Some(msg) = stream.get() {
                        return (Some(Event::Received(msg)), state);
                    } 
                    select! {
                        to_send = msg_recv.recv() => {
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
                            let stream = match state {
                                State::Running {stream, ..} => {
                                    stream
                                }
                                _ => unreachable!()
                            };
                            return (
                                Some(Event::Closed {
                                    stream,
                                }),
                                State::Closed,
                            )
                        }
                        res = stream.update_loop() => {
                            match res {
                                Ok(true) => {
                                    if let Some(msg) = stream.get() {
                                        return (Some(Event::Received(msg)), state);
                                    } else {
                                        unreachable!();
                                    }
                                }
                                Ok(_) => unreachable!(),
                                Err(e) => {
                                    return (
                                        Some(Event::Error(e.into())),
                                        State::Closed
                                    )
                                }
                            }
                        }
                    }
                    (None, state)
                }
                State::Closed => (None, State::Closed),
            }
        },
    )
}
