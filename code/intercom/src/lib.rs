extern crate aareocams_core;
#[cfg(feature="esp32")]
extern crate esp_idf_hal;
#[cfg(feature="rppi")]
extern crate rppal;

#[cfg(feature="esp32")]
pub mod esp32;

#[cfg(feature="rppi")]
pub mod rppi;