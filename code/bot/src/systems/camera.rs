use aareocams_net::{Message, VideoStreamAction};

use async_trait::async_trait;
use dabus::{
    event::EventType,
    stop::{EventActionType, EventArgs},
    BusInterface, BusStop,
    util::GeneralRequirements,
    decl_event,
};
use uuid::Uuid;

use crate::camera_server::CameraServer;

#[derive(Debug)]
pub enum CameraAction {
    FeedCtrlMessage((Uuid, VideoStreamAction)),
    GetReceiver(()),
}

decl_event!(pub, FEED_CTRL_MSG, CameraAction, FeedCtrlMessage, (Uuid, VideoStreamAction), (),                       Some(()), EventType::Send);
decl_event!(pub, GET_RECEIVER,  CameraAction, GetReceiver,     (),                        flume::Receiver<Message>, None,     EventType::Query);


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
}

#[async_trait]
impl BusStop for CameraSystem {
    type Event = CameraAction;

    async fn event<'a>(
        &mut self,
        event: EventArgs<'a, Self::Event>,
        _etype: EventType,
        _bus: BusInterface,
    ) -> Option<Box<dyn GeneralRequirements + Send + 'static>> {
        match event {
            EventArgs::Consume(CameraAction::FeedCtrlMessage((id, msg))) => {
                self.server.feed_ctrl_msg(id, msg);
                None
            }
            EventArgs::Consume(CameraAction::GetReceiver(())) => {
                Some(Box::new(self.server.get_receiver()))
            }
            _ => unreachable!()
        }
    }

    /// after a type match check, how should this event be handled
    fn action(
        &mut self,
        event: &Self::Event,
    ) -> EventActionType {
        match event {
            CameraAction::FeedCtrlMessage(..) => EventActionType::Consume,
            CameraAction::GetReceiver(..) => EventActionType::Consume,
        }
    }
}
