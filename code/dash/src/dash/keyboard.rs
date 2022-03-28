use iced_native::{Subscription, subscription, event::{Event as IcedEvent, Status}, keyboard::{Event as KeyEvent, KeyCode, Modifiers}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyChange {
    Press,
    Release,
}

#[derive(Debug, Clone)]
pub struct Event {

}

fn handle_event(code: KeyCode, mods: Modifiers, change: KeyChange) -> Option<Event> {
    println!("{:?} {:?} {:?}", code, mods, change);
    None
}

pub fn events() -> Subscription<Event> {
    subscription::events_with(|raw_event: IcedEvent, status: Status| -> Option<Event> {
        if let Status::Captured = status {
            None
        } else {
            match raw_event {
                IcedEvent::Keyboard(kbd_event) => {
                    match kbd_event {
                        KeyEvent::KeyPressed { key_code, modifiers } => {
                            handle_event(key_code, modifiers, KeyChange::Press)
                        }
                        KeyEvent::KeyReleased { key_code, modifiers } => {
                            handle_event(key_code, modifiers, KeyChange::Release)
                        }
                        _ => None
                    }
                }
                _ => None
            }
        }
    })
}