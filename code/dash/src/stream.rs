use aareocams_scomm::{connection, Stream};
use iced_native::subscription::{self, Subscription};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use tokio::{io, net::ToSocketAddrs, select};
use tokio::net::TcpStream;

#[derive(thiserror::Error, Debug)]
pub enum Error<A: ToSocketAddrs> {
    #[error("Failed to queue message to be sent:\n{0}")]
    Queue(#[from] connection::QueueError),
    #[error("Failed to update stream:\n{0}")]
    Update(#[from] connection::StreamUpdateErr),
    #[error("Failed to connect to {0}:\n{1}")]
    Connection(A, io::Error),
    #[error("Write error while flushing connection")]
    Flush(connection::write::UpdateError),
    // these are unrecoverable errors
    #[error("Message sender stream closed before a close event was received!")]
    MessageChannelClosed,
    #[error("Stream controll channel closed before a close event was received!")]
    StreamCtrlChClosed,
}

#[derive(Debug, Clone)]
pub enum StreamControllMsg<A: ToSocketAddrs + Debug> {
    ConnectTo(A),
    Disconnect,
    Flush,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum Event<A: ToSocketAddrs + Debug, M: Serialize + DeserializeOwned + Debug> {
    Error(Error<A>),
    Init {
        msg_send: flume::Sender<M>,
        ctrl_send: flume::Sender<StreamControllMsg<A>>,
    },
    ConnectedTo(A),
    Received(M),
}

enum State<A: ToSocketAddrs + Debug, M: Serialize + DeserializeOwned> {
    Uninitialized,
    Ready {
        msg_recv: flume::Receiver<M>,
        ctrl_recv: flume::Receiver<StreamControllMsg<A>>,
    },
    Running {
        stream: Stream<M, bincode::DefaultOptions>,
        msg_recv: flume::Receiver<M>,
        ctrl_recv: flume::Receiver<StreamControllMsg<A>>,
    },
    UnrecoverableExit,
}

pub fn like_and_subscribe<
    's,
    A: ToSocketAddrs + Clone + Sync + Debug + Send + 'static,
    M: Serialize + DeserializeOwned + Debug + Send + 'static,
>() -> Subscription<Event<A, M>> {
    struct ID;

    subscription::unfold(
        std::any::TypeId::of::<ID>(),
        State::Uninitialized,
        move |mut state: State<A, M>| async move {
            match state {
                State::Uninitialized => {
                    let (ctrl_send, ctrl_recv) = flume::unbounded();
                    let (msg_tx, msg_rx) = flume::unbounded();
                    (
                        Some(Event::Init {
                            msg_send: msg_tx,
                            ctrl_send,
                        }),
                        State::Ready {
                            msg_recv: msg_rx,
                            ctrl_recv,
                        },
                    )
                }
                State::Ready {
                    msg_recv,
                    ctrl_recv,
                } => {
                    if let Ok(msg) = ctrl_recv.recv_async().await {
                        match msg {
                            StreamControllMsg::ConnectTo(addr) => {
                                let stream = match TcpStream::connect(addr.clone()).await {
                                    Ok(s) => s,
                                    Err(e) => {
                                        return (
                                            Some(Event::Error(Error::Connection(addr.clone(), e))),
                                            State::Ready {
                                                ctrl_recv,
                                                msg_recv,
                                            },
                                        );
                                    }
                                };
                                let connection = Stream::<M, bincode::DefaultOptions>::new(
                                    stream,
                                    bincode::DefaultOptions::new(),
                                );
                                (
                                    Some(Event::ConnectedTo(addr)),
                                    State::Running {
                                        stream: connection,
                                        msg_recv,
                                        ctrl_recv,
                                    },
                                )
                            }
                            StreamControllMsg::Disconnect => {
                                warn!("Attempted to disconnect, but was not connected");
                                return (
                                    None,
                                    State::Ready {
                                        ctrl_recv,
                                        msg_recv,
                                    },
                                );
                            }
                            StreamControllMsg::Flush => {
                                warn!("Attempted to flush, but was not connected!");
                                return (
                                    None,
                                    State::Ready {
                                        ctrl_recv,
                                        msg_recv,
                                    },
                                );
                            }
                        }
                    } else {
                        return (
                            Some(Event::Error(Error::StreamCtrlChClosed)),
                            State::UnrecoverableExit,
                        );
                    }
                }
                State::Running {
                    ref mut stream,
                    ref mut msg_recv,
                    ref mut ctrl_recv,
                } => {
                    if let Some(msg) = stream.get() {
                        return (Some(Event::Received(msg)), state);
                    }
                    select! {
                        to_send = msg_recv.recv_async() => {
                            if let Ok(msg) = to_send {
                                if let Err(e) = stream.queue(&msg) {
                                    if let State::Running {msg_recv, ctrl_recv, ..} = state {
                                        return (
                                            Some(Event::Error(e.into())),
                                            State::Ready {
                                                msg_recv,
                                                ctrl_recv,
                                            },
                                        )
                                    }
                                    unreachable!()
                                }
                            } else {
                                return (
                                    Some(Event::Error(Error::MessageChannelClosed)),
                                    State::UnrecoverableExit
                                )
                            }
                        }
                        ctrl_update = ctrl_recv.recv_async() => {
                            if let Ok(update) = ctrl_update {
                                match update {
                                    StreamControllMsg::ConnectTo(..) => {
                                        warn!("Attempted to connect while already connected");
                                        return (
                                            None,
                                            state,
                                        )
                                    }
                                    StreamControllMsg::Disconnect => {
                                        if let State::Running { msg_recv, ctrl_recv, .. } = state {
                                            //TODO make this shutdown the stream more nicely
                                            return (
                                                None,
                                                State::Ready { msg_recv, ctrl_recv }
                                            )
                                        } else {
                                            unreachable!()
                                        }
                                    }
                                    StreamControllMsg::Flush => {

                                        if let Err(e) = stream.write_all().await {
                                            if let State::Running { msg_recv, ctrl_recv, .. } = state {
                                                return (
                                                    Some(Event::Error(Error::Flush(e))),
                                                    State::Ready { msg_recv, ctrl_recv }
                                                )
                                            } else {
                                                unreachable!()
                                            }
                                        }
                                    }
                                }
                            } else {
                                return (
                                    Some(Event::Error(Error::StreamCtrlChClosed)),
                                    State::UnrecoverableExit
                                )
                            }
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
                                    if let State::Running {msg_recv, ctrl_recv, ..} = state {
                                        return (
                                            Some(Event::Error(e.into())),
                                            State::Ready {
                                                msg_recv,
                                                ctrl_recv,
                                            },
                                        )
                                    }
                                    unreachable!()
                                }
                            }
                        }
                    }
                    (None, state)
                }
                State::UnrecoverableExit => (None, State::UnrecoverableExit),
            }
        },
    )
}
