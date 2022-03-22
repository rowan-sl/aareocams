use aareocams_net::Message;
use aareocams_scomm::connection::ConnectionReadHalf;
use aareocams_scomm::connection::ConnectionRecvError;
use iced_native::subscription::{self, Subscription};

#[derive(thiserror::Error, Debug)]
pub enum ConnectionError {
    #[error("Failed to receive message {0}")]
    RecvErr(#[from] ConnectionRecvError),
}

#[derive(Debug)]
pub enum ConnectionWatcherEvent {
    Error(ConnectionError),
    MessageReceived(Message),
}

pub enum ConnectionWatcherState {
    Running {
        stream: ConnectionReadHalf<Message, bincode::DefaultOptions>,
    },
    Errored,
}

/// this is stupid, but iced subscriptions are stupid, so its even
pub fn watch(
    receiver: Option<ConnectionReadHalf<Message, bincode::DefaultOptions>>,
) -> Subscription<ConnectionWatcherEvent> {
    struct ConnectionWatcher;

    match receiver {
        Some(stream) => subscription::unfold(
            std::any::TypeId::of::<ConnectionWatcher>(),
            ConnectionWatcherState::Running { stream },
            |r_state| async move {
                match r_state {
                    ConnectionWatcherState::Running { mut stream } => {
                        match stream.recv().await {
                            Err(e) => {
                                return (
                                    Some(ConnectionWatcherEvent::Error(e.into())),
                                    ConnectionWatcherState::Errored,
                                )
                            }
                            _ => {}
                        }
                        if let Some(message) = stream.get() {
                            return (
                                Some(ConnectionWatcherEvent::MessageReceived(message)),
                                ConnectionWatcherState::Running { stream },
                            );
                        }
                        (None, ConnectionWatcherState::Running { stream })
                    }
                    ConnectionWatcherState::Errored => (None, ConnectionWatcherState::Errored),
                }
            },
        ),
        None => subscription::unfold(
            std::any::TypeId::of::<ConnectionWatcher>(),
            (),
            |_| async move { (None, ()) },
        ),
    }
}
