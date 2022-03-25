use bincode::Options as BincodeOptions;
use bytes::BytesMut;
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use std::{collections::VecDeque, marker::PhantomData};


#[derive(Debug, thiserror::Error)]
#[error("Failed to deserialize message:\n{e}")]
pub struct UpdateReaderError {
    #[from]
    e: bincode::Error,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Reader<M: Serialize + DeserializeOwned, O: BincodeOptions + Clone> {
    buf: BytesMut,
    next_msg_len: Option<u64>,
    received: VecDeque<M>,
    #[derivative(Debug="ignore")]
    opts: O,
}

impl<M: Serialize + DeserializeOwned, O: BincodeOptions + Clone> Reader<M, O> {
    pub fn new(opts: O) -> Self {
        Self {
            buf: BytesMut::new(),
            next_msg_len: None,
            received: VecDeque::new(),
            opts,
        }
    }

    pub fn as_byte_sink<'s>(&'s mut self) -> &'s mut BytesMut {
        &mut self.buf
    }

    pub fn buf_len(&self) -> usize {
        self.buf.len()
    }

    fn decode(&mut self) -> Result<bool, UpdateReaderError> {
        let msg_len = usize::try_from(self.next_msg_len.unwrap()).unwrap();

        if self.buf.len() >= msg_len {
            let data = self.buf.split_to(msg_len).freeze();
            let msg = self.opts.clone().deserialize(&data)?;
            self.received.push_front(msg);
            self.next_msg_len = None;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn update(&mut self) -> Result<bool, UpdateReaderError> {
        if let Some(..) = self.next_msg_len {
            self.decode()
        } else {
            const HEADER_SIZE: usize = (u64::BITS / 8) as usize;

            if self.buf.len() >= HEADER_SIZE {
                let data: [u8; HEADER_SIZE] = <dyn std::ops::Deref<Target = [u8]>>::deref(
                    &self.buf.split_to(HEADER_SIZE).freeze(),
                )
                    .try_into()
                    .unwrap();
                let size = u64::from_be_bytes(data);
                self.next_msg_len = Some(size);
                self.update()
            } else {
                Ok(false)
            }
        }
    }

    pub fn full_update(&mut self) -> Result<bool, UpdateReaderError> {
        let mut message_read = false;
        while self.update()? {
            message_read = true;
        }
        Ok(message_read)
    }

    pub fn get_next(&mut self) -> Option<M> {
        self.received.pop_back()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to serialize message:\n{e}")]
pub struct WriterSinkErr {
    #[from]
    e: bincode::Error,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Writer<M: Serialize + DeserializeOwned, O: BincodeOptions + Clone> {
    buf: BytesMut,
    #[derivative(Debug="ignore")]
    opts: O,
    #[derivative(Debug="ignore")]
    _m: PhantomData<M>,
}

impl<M: Serialize + DeserializeOwned, O: BincodeOptions + Clone> Writer<M, O> {
    pub fn new(opts: O) -> Self {
        Self {
            buf: BytesMut::new(),
            opts,
            _m: PhantomData,
        }
    }

    pub fn as_byte_source<'s>(&'s mut self) -> &'s mut BytesMut {
        &mut self.buf
    }

    pub fn buf_len(&self) -> usize {
        self.buf.len()
    }

    pub fn sink(&mut self, m: &M) -> Result<(), WriterSinkErr> {
        let bytes = self.opts.clone().serialize(m)?;
        let msg_len_bytes = (bytes.len() as u64).to_be_bytes();
        self.buf.extend_from_slice(&msg_len_bytes);
        self.buf.extend_from_slice(&bytes);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestMessage {
    foo: usize,
    bar: String,
}

#[test]
fn test_ser_deser() {
    let msg = TestMessage {
        foo: 10,
        bar: "Hello, world!".to_string(),
    };

    let mut reader = Reader::<TestMessage, _>::new(bincode::options());
    let mut writer = Writer::<TestMessage, _>::new(bincode::options());

    writer.sink(&msg).expect("Wrote message to writer");
    *reader.as_byte_sink() = writer.as_byte_source().clone();

    reader.full_update().expect("Updated the reader");
    assert_eq!(Some(msg), reader.get_next());
}

#[test]
fn test_reader_debug() {
    let reader = Reader::<TestMessage, _>::new(bincode::options());
    dbg!(reader);
}

#[test]
fn test_writer_debug() {
    let writer = Writer::<TestMessage, _>::new(bincode::options());
    dbg!(writer);
}
