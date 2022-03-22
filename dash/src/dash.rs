use aareocams_net::Message;
use iced::{Application, Command, Subscription, button::{self, Button}, Text};
use std::fmt::Debug;
use tokio::{
    net::ToSocketAddrs,
    sync::{mpsc, oneshot},
};
use crate::stream;


#[derive(Debug)]
pub enum GUIMsg<A: tokio::net::ToSocketAddrs + Debug> {
    Socket(stream::Event<A, Message>),
    Interaction(Interaction),
}

#[derive(Debug, Clone)]
pub enum Interaction {
    Click
}

pub struct GUIState {
    clicky: button::State,
}

pub struct Dashboard<A>
where
    A: tokio::net::ToSocketAddrs,
{
    /// address used to connect to the bot
    addr: A,
    /// channel to send messages, gets passed on to the socket
    sender_channel: Option<mpsc::UnboundedSender<Message>>,
    /// channel to close the sender
    close_sender: Option<oneshot::Sender<()>>,
    /// the state for all GUI elements
    gui: GUIState,
    exit: bool,
}

impl<A> Application for Dashboard<A>
where
    A: ToSocketAddrs + Clone + Sync + Debug + Send + 'static,
{
    type Message = GUIMsg<A>;
    // type Flags = (Controller, Connection<Message, bincode::DefaultOptions>);
    type Flags = (A, usize);
    type Executor = iced::executor::Default;

    fn new(flags: Self::Flags) -> (Self, Command<GUIMsg<A>>) {
        (
            Self {
                addr: flags.0,
                sender_channel: None,
                close_sender: None,
                gui: GUIState {
                    clicky: button::State::new()
                },
                exit: false,
            },
            Command::none(),
        )
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        stream::like_and_subscribe(self.addr.clone()).map(GUIMsg::Socket)
    }

    fn title(&self) -> String {
        "AAREOCAMS - Dashboard".into()
    }

    fn update(&mut self, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            GUIMsg::Socket(socket_event) => {
                use stream::Event;
                dbg!(&socket_event);

                match socket_event {
                    Event::Init { close_sig_send, msg_send } => {
                        self.sender_channel = Some(msg_send);
                        self.close_sender = Some(close_sig_send);
                    }
                    Event::Error(e) => {
                        eprintln!("Error sending message\n{:#?}", e);
                        self.exit = true;
                    }
                    _ => {},
                }
            }
            GUIMsg::Interaction(interaction_event) => {
                match interaction_event {
                    Interaction::Click => {
                        if let Some(ref mut sender) = self.sender_channel {
                            sender.send(Message::Click).unwrap();
                        }
                    }
                }
                dbg!(interaction_event);
            }
        }
        Command::none()
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let root: iced::Element<Interaction> = iced::Column::new()
            .push(
                Button::new(&mut self.gui.clicky, Text::new("Click me!"))
                    .on_press(Interaction::Click)
            )
            .into();
        root.map(Self::Message::Interaction)
    }
}
