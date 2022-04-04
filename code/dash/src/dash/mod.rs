mod keyboard;

use crate::stream::{self, StreamControllMsg};
use aareocams_core::H264Decoder;
use aareocams_net::Message;
use iced::{
    button::{self, Button},
    Application, Command, Subscription, Text,
};
use image::DynamicImage;
use std::fmt::Debug;
use tokio::{net::ToSocketAddrs, sync::mpsc};

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
}

pub struct GUIState {
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
    /// test feild, remove this for a proper interface later
    video_0_handle: iced::image::Handle,
    video_0_decoder: H264Decoder,
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
                    connect: button::State::new(),
                    disconnect: button::State::new(),
                },
                video_0_handle: iced::image::Handle::from_pixels(0, 0, vec![]),
                video_0_decoder: H264Decoder::new().unwrap(),
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
                    Event::Received(message) => {
                        match message {
                            Message::DashboardDisconnect => unreachable!(),
                            Message::VideoStream { stream_id: _, dimensions, data } => {
                                match self.video_0_decoder.decode(&data) {
                                    Ok(mut decoded) => {
                                        if let Some(img) = decoded.pop() {
                                            let bgra_img = DynamicImage::ImageRgb8(img).to_bgra8();
                                            self.video_0_handle = iced::image::Handle::from_pixels(dimensions.0, dimensions.1, bgra_img.into_vec());
                                        }
                                    }
                                    Err(e) => {
                                        error!("Decoding error: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                    Event::ConnectedTo(_addr) => {}
                }
            }
            GUIMsg::Interaction(interaction_event) => {
                match interaction_event {
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
                }
                dbg!(interaction_event);
            }
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
            .push(
                iced::image::Image::new(self.video_0_handle.clone())
            )
            .into();
        root.map(Self::Message::Interaction)
    }
}
