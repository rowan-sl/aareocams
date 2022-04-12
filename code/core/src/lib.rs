extern crate image;
extern crate openh264;

use std::ops::Deref;
use image::RgbImage;
use openh264::{nal_units, encoder::{Encoder, EncoderConfig}, decoder::Decoder};

pub struct H264Encoder {
    encoder: Encoder,
    converter: openh264::formats::RBGYUVConverter,
}

impl H264Encoder {
    pub fn new(width: u32, height: u32) -> Result<Self, openh264::Error> {
        let encoder = Encoder::with_config(EncoderConfig::new(width, height))?;
        let converter = openh264::formats::RBGYUVConverter::new(width as usize, height as usize);
        Ok(Self {
            encoder,
            converter,
        })
    }

    pub fn encode(&mut self, img: &RgbImage) -> Result<Vec<u8>, openh264::Error> {
        self.converter.convert(img);
        Ok(self.encoder.encode(&self.converter)?.to_vec())
    }
}

pub struct H264Decoder {
    decoder: Decoder,
    buffer: Vec<u8>,
}

impl H264Decoder {
    pub fn new() -> Result<Self, openh264::Error> {
        Ok(Self {
            decoder: Decoder::with_config(openh264::decoder::DecoderConfig::new().debug(true))?,
            buffer: vec![],
        })
    }

    pub fn decode(&mut self, data: &[u8]) -> Result<Vec<RgbImage>, openh264::Error> {
        let mut decoded_images = vec![];

        self.buffer.extend_from_slice(data);

        let mut error = None;

        let mut removed_data = 0usize;
        for packet in nal_units(&self.buffer) {
            removed_data += packet.len();
            let decoded = match self.decoder.decode(packet) {
                Ok(dec) => dec,
                Err(e) => {
                    error = Some(e);
                    break;
                }
            };
            let dim = decoded.dimension_rgb();
            if dim != (0, 0) {
                let rgb_len = dim.0 * dim.1 * 3;
                let mut raw_rgb = vec![0; rgb_len];
                decoded.write_rgb8(&mut raw_rgb)?;
                decoded_images.push(RgbImage::from_raw(dim.0 as u32, dim.1 as u32, raw_rgb).unwrap());
            }
        }

        self.buffer.drain(0..removed_data);

        if let Some(err) = error {
            Err(err)
        } else {
            Ok(decoded_images)
        }
    }
}

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
