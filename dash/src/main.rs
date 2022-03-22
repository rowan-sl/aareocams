extern crate aareocams_net;
extern crate aareocams_scomm;
extern crate aareocams_core;
extern crate anyhow;
extern crate bincode;
extern crate serde;
extern crate sn30pro;
extern crate tokio;
extern crate iced;
extern crate iced_native;
extern crate thiserror;

mod stream_watcher;
mod stream_sender;

use std::cell::RefCell;

use aareocams_net::Message;
use aareocams_scomm::{Connection, connection::{ConnectionWriteHalf, ConnectionReadHalf}};
use anyhow::Result;
use iced::{Application, Command, Subscription, Settings};
use serde::{de::DeserializeOwned, Serialize};
use sn30pro::Controller;
use tokio::{net::TcpStream, sync::{mpsc, oneshot}};

mod config {
    pub const ADDR: &str = "127.0.0.1:6440";
}

struct InitRes<M: Serialize + DeserializeOwned, O: bincode::Options + Copy> {
    controlls: Controller,
    connection: Connection<M, O>,
}

async fn init() -> Result<InitRes<Message, bincode::DefaultOptions>> {
    let controlls = Controller::new(0).await?;

    let raw_socket = TcpStream::connect(config::ADDR).await?;
    let connection = Connection::<Message, _>::new(raw_socket, bincode::DefaultOptions::new());

    Ok(InitRes {
        controlls,
        connection
    })
}

fn main() -> Result<()> {
    let InitRes {
        controlls,
        connection
    } = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(init())?;
    
    Dashboard::run(Settings::with_flags(
        (controlls, connection)
    ))?;

    Ok(())
}

#[derive(Debug)]
enum DashboardGUIMessage {
    Socket(stream_watcher::ConnectionWatcherEvent),
    Sender(stream_sender::ConnectionSenderEvent),
    Interaction(Interaction),
}

#[derive(Debug, Clone)]
enum Interaction {
}

struct Dashboard {
    controlls: Controller,
    // behold: the cursed combo
    write_half: RefCell<Option<ConnectionWriteHalf<Message, bincode::DefaultOptions>>>,
    read_half: RefCell<Option<ConnectionReadHalf<Message, bincode::DefaultOptions>>>,
    /// channel to send messages, gets passed on to the socket
    sender_channel: Option<mpsc::UnboundedSender<Message>>,
    /// channel to close the sender
    close_sender: Option<oneshot::Sender<()>>,
    exit: bool,
}

impl Application for Dashboard {
    type Message = DashboardGUIMessage;
    type Flags = (Controller, Connection<Message, bincode::DefaultOptions>);
    type Executor = iced::executor::Default;

    fn new(flags: Self::Flags) -> (Self, Command<DashboardGUIMessage>) {
        let (read_half, write_half) = flags.1.into_split();
        (
            Self {
                controlls: flags.0,
                write_half: RefCell::new(Some(write_half)),
                read_half: RefCell::new(Some(read_half)),
                sender_channel: None,
                close_sender: None,
                exit: false
            },
            // Command::perform(async move {Interaction::A}, |a| DashboardGUIMessage::Interaction(a))
            Command::none()
        )
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // get GOTTEN, iced
        // *angry borrow checker noises*
        Subscription::batch(vec![
            stream_watcher::watch(self.read_half.borrow_mut().take()).map(DashboardGUIMessage::Socket),
            stream_sender::init_sender(self.write_half.borrow_mut().take()).map(DashboardGUIMessage::Sender),
        ])
    }

    fn title(&self) -> String {
        "AAREOCAMS - Dashboard".into()
    }

    fn update(&mut self, msg: Self::Message) -> Command<Self::Message> {
        use DashboardGUIMessage as GUIMsg;
        use stream_sender::ConnectionSenderEvent;
        use stream_watcher::ConnectionWatcherEvent;

        match msg {
            GUIMsg::Socket(_socket_event) => {
                dbg!(_socket_event);
                todo!()
            }
            GUIMsg::Interaction(_interaction_event) => {
                dbg!(_interaction_event);
                todo!()
            }
            GUIMsg::Sender(sender_event) => {
                //TODO handle init events
                dbg!(&sender_event);
                match sender_event {
                    ConnectionSenderEvent::Init { close_sig, send_channel } => {
                        self.sender_channel = Some(send_channel);
                        self.close_sender = Some(close_sig);
                    }
                    _ => todo!()
                }
            }
        }
        Command::none()
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let root: iced::Element<Interaction> = 
            iced::Column::new()
                .into();
        root.map(Self::Message::Interaction)
    }
}
