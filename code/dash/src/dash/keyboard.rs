use aareocams_net::DriveAction;
use iced_native::{
    event::{Event as IcedEvent, Status},
    keyboard::{Event as KeyEvent, KeyCode, Modifiers},
    subscription, Subscription,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyChange {
    Press,
    Release,
}

#[derive(Debug, Clone)]
pub enum Event {
    Drive(DriveAction),
}

fn handle_event(code: KeyCode, mods: Modifiers, change: KeyChange) -> Option<Event> {
    println!("{:?} {:?} {:?}", code, mods, change);
    match (code, mods, change) {
        (KeyCode::W, _, KeyChange::Press) => return Some(Event::Drive(DriveAction::Fwd)),
        (KeyCode::S, _, KeyChange::Press) => return Some(Event::Drive(DriveAction::Rev)),
        (KeyCode::A, _, KeyChange::Press) => return Some(Event::Drive(DriveAction::Stop)),
        _ => None,
    }
}

pub fn events() -> Subscription<Event> {
    subscription::events_with(|raw_event: IcedEvent, status: Status| -> Option<Event> {
        if let Status::Captured = status {
            None
        } else {
            match raw_event {
                IcedEvent::Keyboard(kbd_event) => match kbd_event {
                    KeyEvent::KeyPressed {
                        key_code,
                        modifiers,
                    } => handle_event(key_code, modifiers, KeyChange::Press),
                    KeyEvent::KeyReleased {
                        key_code,
                        modifiers,
                    } => handle_event(key_code, modifiers, KeyChange::Release),
                    _ => None,
                },
                _ => None,
            }
        }
    })
}
