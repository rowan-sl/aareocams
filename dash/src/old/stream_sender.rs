use aareocams_net::Message;
use aareocams_scomm::connection::ConnectionSendError;
use aareocams_scomm::connection::ConnectionWriteHalf;
use iced_native::subscription::{self, Subscription};
use tokio::select;
use tokio::sync::{mpsc, oneshot};

#[derive(thiserror::Error, Debug)]
pub enum ConnectionError {
    #[error("Failed to send message {0}")]
    RecvErr(#[from] ConnectionSendError),
    #[error("Failed to serialize message {0}")]
    SeriErr(#[from] bincode::Error),
}

#[derive(Debug)]
pub enum ConnectionSenderEvent {
    Error(ConnectionError),
    Init {
        /// closes the channel, sending all pending data and returning the ConnectionWriteHalf
        close_sig: oneshot::Sender<()>,
        send_channel: mpsc::UnboundedSender<Message>,
    },
    Closed {
        stream: ConnectionWriteHalf<Message, bincode::DefaultOptions>,
    },
}

pub struct _ConnectionSenderState {
    pub stream: ConnectionWriteHalf<Message, bincode::DefaultOptions>,
    pub recv_channel: mpsc::UnboundedReceiver<Message>,
    pub close_recv: oneshot::Receiver<()>,
}

pub enum ConnectionSenderState {
    NeedsInit(ConnectionWriteHalf<Message, bincode::DefaultOptions>),
    Running(_ConnectionSenderState),
    Closed,
}

/// this is stupid, but iced subscriptions are stupid, so its even
pub fn init_sender(
    sender: Option<ConnectionWriteHalf<Message, bincode::DefaultOptions>>,
) -> Subscription<ConnectionSenderEvent> {
    struct ConnectionSender;

    match sender {
        Some(stream) => {
            subscription::unfold(
                std::any::TypeId::of::<ConnectionSender>(),
                ConnectionSenderState::NeedsInit(stream),
                |r_state| async move {
                    match r_state {
                        ConnectionSenderState::NeedsInit(stream) => {
                            let (close_send, close_recv) = oneshot::channel();
                            let (send_channel, recv_channel) = mpsc::unbounded_channel();
                            (
                                Some(ConnectionSenderEvent::Init {
                                    close_sig: close_send,
                                    send_channel,
                                }),
                                ConnectionSenderState::Running(_ConnectionSenderState {
                                    stream,
                                    recv_channel,
                                    close_recv,
                                }),
                            )
                        }
                        ConnectionSenderState::Running(mut state) => {
                            //TODO implement message sending
                            if let Err(e) = state.stream.send_all().await {
                                return (
                                    Some(ConnectionSenderEvent::Error(e.into())),
                                    ConnectionSenderState::Closed,
                                );
                            }

                            select! {
                                _ = &mut state.close_recv => {
                                    return (
                                        Some(ConnectionSenderEvent::Closed{stream: state.stream}),
                                        ConnectionSenderState::Closed,
                                    )
                                }
                                to_send = state.recv_channel.recv() => {
                                    match to_send {
                                        Some(msg) => {
                                            if let Err(e) = state.stream.queue(&msg) {
                                                return (
                                                    Some(ConnectionSenderEvent::Error(e.into())),
                                                    ConnectionSenderState::Closed,
                                                )
                                            }
                                        }
                                        None => {
                                            return (
                                                Some(ConnectionSenderEvent::Closed{stream: state.stream}),
                                                ConnectionSenderState::Closed,
                                            )
                                        }
                                    }
                                }
                            }

                            (None, ConnectionSenderState::Running(state))
                        }
                        ConnectionSenderState::Closed => (None, ConnectionSenderState::Closed),
                    }
                },
            )
        }
        None => subscription::unfold(
            std::any::TypeId::of::<ConnectionSender>(),
            (),
            |_| async move { (None, ()) },
        ),
    }
}
