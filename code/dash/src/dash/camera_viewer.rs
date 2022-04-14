use aareocams_net::{Message, VideoStreamAction};
use iced::{
    button,
    image::{Handle as IcedImageHandle, Image as IcedImage},
    text_input, Alignment, Button, Column, Length, Row, Text, TextInput,
};
use image::DynamicImage;
use lvenc::Decoder;
use lvenc::Packet;
use uuid::Uuid;

#[derive(Debug)]
pub struct VideoStream {
    pub decoder: Decoder,
    pub stream_id: Uuid,
    pub image_handle: IcedImageHandle,
    pub pause_btn: button::State,
    pub resume_btn: button::State,
    pub close_btn: button::State,
    pub paused: bool,
}

#[derive(Debug, Clone)]
pub enum CameraViewerEvent {
    /// pause stream with some id
    Pause(Uuid),
    /// resume stream with some id
    Resume(Uuid),
    /// close stream with some id
    Close(Uuid),
    StreamIDInputChange(String),
    CreateStream,
}

#[derive(Debug)]
pub struct CameraViewer {
    new_stream_input_state: text_input::State,
    new_stream_input_text: String,
    new_stream_btn_state: button::State,
    streams: Vec<VideoStream>,
    messages: Vec<Message>,
}

impl CameraViewer {
    fn stream_by_id(&mut self, id: Uuid) -> Option<&mut VideoStream> {
        for stream in &mut self.streams {
            if id == stream.stream_id {
                return Some(stream);
            }
        }
        None
    }

    pub fn new() -> Self {
        Self {
            new_stream_input_state: text_input::State::new(),
            new_stream_input_text: String::new(),
            new_stream_btn_state: button::State::new(),
            streams: vec![],
            messages: vec![],
        }
    }

    pub fn register_stream(&mut self, id: Uuid) {
        self.streams.push(VideoStream {
            decoder: Decoder::new(),
            stream_id: id,
            image_handle: IcedImageHandle::from_pixels(0, 0, vec![]),
            pause_btn: button::State::new(),
            resume_btn: button::State::new(),
            close_btn: button::State::new(),
            paused: false,
        });
    }

    pub fn close_stream(&mut self, id: Uuid) {
        self.streams.drain_filter(move |i| i.stream_id == id);
    }

    pub fn view(&mut self) -> iced::Element<CameraViewerEvent> {
        let controlls = Column::<'_, CameraViewerEvent>::new()
            .height(Length::Shrink)
            .width(Length::Shrink)
            .spacing(2)
            .push(
                TextInput::new(
                    &mut self.new_stream_input_state,
                    "id",
                    &self.new_stream_input_text,
                    |new| CameraViewerEvent::StreamIDInputChange(new),
                )
                .width(Length::Units(100)),
            )
            .push(
                Button::new(&mut self.new_stream_btn_state, Text::new("Connect stream"))
                    .on_press(CameraViewerEvent::CreateStream),
            );

        let mut root_children = vec![];

        root_children.push(controlls.into());

        for cam in &mut self.streams {
            root_children.push(
                Column::new()
                    .height(Length::Shrink)
                    .width(Length::Shrink)
                    .spacing(5)
                    .push(
                        IcedImage::new(cam.image_handle.clone())
                            .height(Length::Shrink)
                            .content_fit(iced::ContentFit::Contain),
                    )
                    .push(
                        Row::new()
                            .align_items(Alignment::Center)
                            .height(Length::Shrink)
                            .spacing(2)
                            .push(
                                Button::new(&mut cam.pause_btn, Text::new("Pause"))
                                    .on_press(CameraViewerEvent::Pause(cam.stream_id)),
                            )
                            .push(
                                Button::new(&mut cam.resume_btn, Text::new("Resume"))
                                    .on_press(CameraViewerEvent::Resume(cam.stream_id)),
                            )
                            .push(
                                Button::new(&mut cam.close_btn, Text::new("Close"))
                                    .on_press(CameraViewerEvent::Close(cam.stream_id)),
                            ),
                    )
                    .into(),
            );
        }

        let root: iced::Element<CameraViewerEvent> =
            Row::<'_, CameraViewerEvent>::with_children(root_children)
                .align_items(Alignment::Center)
                .padding(5)
                .spacing(2)
                .height(Length::Shrink)
                .width(Length::Shrink)
                .into();

        // root.explain([255.0, 0.0, 0.0])
        root
    }

    pub fn feed_event(&mut self, event: CameraViewerEvent) {
        match event {
            CameraViewerEvent::Pause(id) => {
                if let Some(stream) = self.stream_by_id(id) {
                    if !stream.paused {
                        stream.paused = true;
                        self.messages.push(Message::VideoStreamCtl {
                            id,
                            action: VideoStreamAction::Pause,
                        });
                    }
                }
            }
            CameraViewerEvent::Resume(id) => {
                if let Some(stream) = self.stream_by_id(id) {
                    if stream.paused {
                        stream.paused = false;
                        self.messages.push(Message::VideoStreamCtl {
                            id,
                            action: VideoStreamAction::Resume,
                        });
                    }
                }
            }
            CameraViewerEvent::Close(id) => {
                self.close_stream(id);
                self.messages.push(Message::VideoStreamCtl {
                    id,
                    action: VideoStreamAction::Close,
                });
            }
            CameraViewerEvent::CreateStream => {
                if let Ok(id) = self.new_stream_input_text.parse::<usize>() {
                    let uuid = Uuid::new_v4();
                    self.new_stream_input_text.clear();
                    self.register_stream(uuid);
                    self.messages.push(Message::VideoStreamCtl {
                        id: uuid,
                        action: VideoStreamAction::Init { dev: id },
                    });
                }
            }
            CameraViewerEvent::StreamIDInputChange(new) => {
                self.new_stream_input_text = new;
            }
        }
    }

    pub fn messages(&mut self) -> &mut Vec<Message> {
        &mut self.messages
    }

    pub fn feed_message(&mut self, id: Uuid, packet: Packet) {
        if let Some(stream) = self.stream_by_id(id) {
            stream.decoder.feed_packet(packet);
            if let Some(next_frame) = stream.decoder.frames().last() {
                let bgr = DynamicImage::ImageRgb8(next_frame).into_bgra8();
                stream.image_handle =
                    IcedImageHandle::from_pixels(bgr.width(), bgr.height(), bgr.to_vec());
            }
        }
    }
}
