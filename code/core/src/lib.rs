use std::ops::Deref;


/// A [`bool`] that can be set to true only **once**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fuse {
    burnt: bool,
}

impl Fuse {
    pub const fn new() -> Self {
        Self { burnt: false }
    }

    pub fn burn(&mut self) {
        self.burnt = true
    }
}

impl Deref for Fuse {
    type Target = bool;

    fn deref(&self) -> &bool {
        &self.burnt
    }
}
