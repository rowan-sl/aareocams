use std::marker::PhantomData;

use serde::Serialize;

pub type Error = bincode::Error;

const RANDOM_NESS: [u8; 69] =
    *b"alksjdlfi2h3uinqiu3498hgqi3rkbh3 miuhqr9g8 94uq912423562345yyjety[[]a";

#[derive(Debug, thiserror::Error)]
pub enum DecodeHeaderError {
    #[error("Invalid data size! (expected {0} found {1})")]
    InvalidSize(usize, usize),
    #[error("Invalid validation content!")]
    InvalidContent,
}

#[derive(Debug, Default)]
pub struct Header<M: Serialize, O: bincode::Options + Copy> {
    size: u64,
    options: O,
    _msg_type: PhantomData<M>,
}

impl<M: Serialize, O: bincode::Options + Copy> Clone for Header<M, O> {
    fn clone(&self) -> Self {
        Self {
            size: self.size,
            options: self.options,
            _msg_type: PhantomData,
        }
    }
}

impl<M: Serialize, O: bincode::Options + Copy> Header<M, O> {
    #[must_use]
    pub fn header_for(value: &M, options: O) -> Result<Self, Error> {
        Ok(Self {
            size: options.serialized_size(value)?,
            options,
            _msg_type: PhantomData,
        })
    }

    #[must_use]
    pub fn header_byte_size() -> usize {
        RANDOM_NESS.len() + std::mem::size_of::<u64>()
    }

    #[must_use]
    pub fn decode_from_bytes(bytes: &[u8], opts: O) -> Result<Header<M, O>, DecodeHeaderError> {
        if bytes.len() != Header::<M, O>::header_byte_size() {
            Err(DecodeHeaderError::InvalidSize(
                Header::<M, O>::header_byte_size(),
                bytes.len(),
            ))
        } else {
            if bytes[0..69] == RANDOM_NESS[..] {
                let mut data: [u8; 8] = [0; 8];
                data.copy_from_slice(&bytes[69..]);
                Ok(Header {
                    size: u64::from_be_bytes(data),
                    options: opts,
                    _msg_type: PhantomData,
                })
            } else {
                Err(DecodeHeaderError::InvalidContent)
            }
        }
    }

    #[must_use]
    pub fn into_raw_parts(self) -> (u64, O) {
        let Header {
            size,
            options,
            _msg_type,
        } = self;
        (size, options)
    }

    #[must_use]
    pub fn size(&self) -> u64 {
        self.size
    }

    #[must_use]
    pub fn serialize_header(&self) -> Vec<u8> {
        let mut b = vec![];
        b.extend_from_slice(&RANDOM_NESS);
        b.extend_from_slice(&self.size.to_be_bytes());
        b
    }
}
