mod keyboard;

use crate::stream::{self, StreamControllMsg};
use aareocams_net::Message;
use iced::{
    button::{self, Button},
    Application, Command, Subscription, Text,
};
use std::fmt::Debug;
use tokio::{
    net::ToSocketAddrs,
    sync::mpsc,
};

#[derive(Debug)]
pub enum GUIMsg<A: tokio::net::ToSocketAddrs + Debug> {
    Socket(stream::Event<A, Message>),
    Keyboard(keyboard::Event),
    Interaction(Interaction),
}

#[derive(Debug, Clone)]
pub enum Interaction {
    Click,
    Connect,
    Disconnect,
}

pub struct GUIState {
    clicky: button::State,
    connect: button::State,
    disconnect: button::State,
}

struct StreamInterface<A: ToSocketAddrs + Debug> {
    /// channel to send messages, gets passed on to the socket
    pub msg_send: mpsc::UnboundedSender<Message>,
    pub ctrl_send: mpsc::UnboundedSender<stream::StreamControllMsg<A>>,
}

pub struct Dashboard<A>
where
    A: tokio::net::ToSocketAddrs + Debug,
{
    addr: A,
    /// holds all communication elements with the stream subscription
    stream: Option<StreamInterface<A>>,
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
                stream: None,
                gui: GUIState {
                    clicky: button::State::new(),
                    connect: button::State::new(),
                    disconnect: button::State::new(),
                },
                exit: false,
            },
            Command::none(),
        )
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            stream::like_and_subscribe().map(GUIMsg::Socket),
            keyboard::events().map(GUIMsg::Keyboard),
        ])
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
                    Event::Init {
                        msg_send,
                        ctrl_send,
                    } => {
                        self.stream = Some(StreamInterface {
                            msg_send,
                            ctrl_send,
                        });
                    }
                    Event::Error(e) => {
                        eprintln!("Error sending message\n{:#?}", e);
                        self.exit = true;
                    }
                    _ => {}
                }
            }
            GUIMsg::Interaction(interaction_event) => {
                match interaction_event {
                    Interaction::Click => {
                        if let Some(ref mut stream) = self.stream {
                            stream.msg_send.send(Message::Click).unwrap();
                        }
                    }
                    Interaction::Connect => {
                        if let Some(ref mut stream) = self.stream {
                            stream.ctrl_send.send(StreamControllMsg::ConnectTo(self.addr.clone())).unwrap();
                        }
                    }
                    Interaction::Disconnect => {
                        if let Some(ref mut stream) = self.stream {
                            stream.msg_send.send(Message::DashboardDisconnect).unwrap();
                            stream.ctrl_send.send(StreamControllMsg::Flush).unwrap();
                            stream.ctrl_send.send(StreamControllMsg::Disconnect).unwrap();
                        }
                    }
                }
                dbg!(interaction_event);
            }
            GUIMsg::Keyboard(_keyboard_event) => {

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
                    .on_press(Interaction::Click),
            )
            .push(
                Button::new(&mut self.gui.connect, Text::new("connect"))
                .on_press(Interaction::Connect),
            )
            .push(
                Button::new(&mut self.gui.disconnect, Text::new("disconnect"))
                .on_press(Interaction::Disconnect),
            )
            .into();
        root.map(Self::Message::Interaction)
    }
}
