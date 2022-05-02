mod camera_viewer;
mod keyboard;

use crate::stream::{self, StreamControllMsg};
use aareocams_net::Message;
use camera_viewer::{CameraViewer, CameraViewerEvent};
use iced::{
    button::{self, Button},
    Application, Command, Subscription, Text,
};
use std::fmt::Debug;
use tokio::net::ToSocketAddrs;

#[derive(Debug)]
pub enum GUIMsg<A: tokio::net::ToSocketAddrs + Debug> {
    Socket(stream::Event<A, Message>),
    Keyboard(keyboard::Event),
    Interaction(Interaction),
}

#[derive(Debug, Clone)]
pub enum Interaction {
    Connect,
    Disconnect,
    CameraStream(CameraViewerEvent),
}

pub struct GUIState {
    connect: button::State,
    disconnect: button::State,
}

struct StreamInterface<A: ToSocketAddrs + Debug> {
    /// channel to send messages, gets passed on to the socket
    pub msg_send: flume::Sender<Message>,
    pub ctrl_send: flume::Sender<stream::StreamControllMsg<A>>,
}

pub struct Dashboard<A>
where
    A: tokio::net::ToSocketAddrs + Debug,
{
    addr: A,
    /// holds all communication elements with the stream subscription
    stream: Option<StreamInterface<A>>,
    streams: CameraViewer,
    /// the state for all GUI elements
    gui: GUIState,
    exit: bool,
}

impl<A> Application for Dashboard<A>
where
    A: ToSocketAddrs + Clone + Sync + Debug + Send + 'static,
{
    type Message = GUIMsg<A>;
    /// (ip addr of bot, controller ID)
    type Flags = (A, usize);
    type Executor = iced::executor::Default;

    fn new(flags: Self::Flags) -> (Self, Command<GUIMsg<A>>) {
        (
            Self {
                addr: flags.0,
                stream: None,
                gui: GUIState {
                    connect: button::State::new(),
                    disconnect: button::State::new(),
                },
                streams: CameraViewer::new(),
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
                // dbg!(&socket_event);

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
                    Event::Received(message) => match message {
                        Message::DashboardDisconnect => unreachable!(),
                        Message::VideoStreamData { id, packet } => {
                            self.streams.feed_message(id, packet);
                        }
                        Message::VideoStreamCtl { .. } => {}
                        Message::VideoStreamInfo { id, action } => {
                            //TODO properly handle video stream info messages
                            info!("VideoStreamInfo: {}: {:?}", id, action);
                        }
                    },
                    Event::ConnectedTo(_addr) => {}
                }
            }
            GUIMsg::Interaction(interaction_event) => match interaction_event {
                Interaction::Connect => {
                    if let Some(ref mut stream) = self.stream {
                        stream
                            .ctrl_send
                            .send(StreamControllMsg::ConnectTo(self.addr.clone()))
                            .unwrap();
                    }
                }
                Interaction::Disconnect => {
                    if let Some(ref mut stream) = self.stream {
                        stream.msg_send.send(Message::DashboardDisconnect).unwrap();
                        stream.ctrl_send.send(StreamControllMsg::Flush).unwrap();
                        stream
                            .ctrl_send
                            .send(StreamControllMsg::Disconnect)
                            .unwrap();
                    }
                }
                Interaction::CameraStream(event) => {
                    self.streams.feed_event(event);
                    for message in self.streams.messages().drain(..) {
                        self.stream
                            .as_ref()
                            .unwrap()
                            .msg_send
                            .send(message)
                            .unwrap();
                    }
                }
            },
            GUIMsg::Keyboard(_keyboard_event) => {}
        }
        Command::none()
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let root: iced::Element<Interaction> = iced::Column::new()
            .push(
                Button::new(&mut self.gui.connect, Text::new("connect"))
                    .on_press(Interaction::Connect),
            )
            .push(
                Button::new(&mut self.gui.disconnect, Text::new("disconnect"))
                    .on_press(Interaction::Disconnect),
            )
            .push(self.streams.view().map(Interaction::CameraStream))
            .into();
        root.map(Self::Message::Interaction)
    }
}
