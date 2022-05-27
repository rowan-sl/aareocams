use aareocams_net::{Message, VideoStreamAction};

use dabus::{
    EventRegister,
    BusInterface, BusStop,
    event,
};
use flume::Receiver;
use uuid::Uuid;

use crate::camera_server::CameraServer;

// decl_event!(pub, FEED_CTRL_MSG, CameraAction, FeedCtrlMessage, (Uuid, VideoStreamAction), (),                       Some(()), EventType::Send);
// decl_event!(pub, GET_RECEIVER,  CameraAction, GetReceiver,     (),                        flume::Receiver<Message>, None,     EventType::Query);
event!(FEED_CTRL_MSG, (Uuid, VideoStreamAction), ());
event!(GET_RECEIVER, (), Receiver<Message>);


#[derive(Debug)]
pub struct CameraSystem {
    server: CameraServer,
}

impl CameraSystem {
    pub fn new() -> Self {
        Self {
            server: CameraServer::new(),
        }
    }

    async fn ctrl_msg(
        &mut self,
        msg: (Uuid, VideoStreamAction),
        _bus: BusInterface,
    ) {
        self.server.feed_ctrl_msg(msg.0, msg.1);
    }

    async fn get_receiver(
        &mut self,
        _: (),
        _bus: BusInterface,
    ) -> Receiver<Message> {
        self.server.get_receiver()
    }
}

impl BusStop for CameraSystem {
    fn registered_handlers(h: EventRegister<Self>) -> EventRegister<Self> {
        h
            .handler(FEED_CTRL_MSG, Self::ctrl_msg)
            .handler(GET_RECEIVER, Self::get_receiver)
    }
}