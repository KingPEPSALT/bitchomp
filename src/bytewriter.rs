use std::mem::size_of;

use super::{Endianness, ToBytes};

#[derive(Debug)]
pub enum ByteWriterError {
    Fail,
}
// T::Bytes: Into<Vec<u8>>
pub trait ByteWriterResource = ToBytes<Bytes = Vec<u8>>;
pub struct ByteWriter {
    buf: Vec<u8>,
    endianness: Endianness,
}

impl ByteWriter {
    pub fn new(endianness: Endianness) -> Self {
        ByteWriter {
            buf: Vec::new(),
            endianness,
        }
    }
    pub fn append<T: ByteWriterResource>(&mut self, data: T) -> usize {
        let mut buf = data.to_bytes(self.endianness);
        self.buf.append(&mut buf);
        buf.len()
    }
    pub fn write<T: ByteWriterResource>(
        &mut self,
        data: T,
        pos: usize,
    ) -> Result<usize, ByteWriterError> {
        let buf = data.to_bytes(self.endianness);
        let size = buf.len();
        for i in 0..size {
            self.buf[pos + i] = buf[i];
        }
        Ok(buf.len())
    }
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn write_vec<T: ByteWriterResource + Clone>(&mut self, data: Vec<T>) -> usize {
        for v in data.iter() {
            self.append::<T>(v.clone());
        }
        data.len() * size_of::<T>()
    }

    pub fn write_sized_vec<T: ByteWriterResource + Clone>(&mut self, data: Vec<T>) -> usize {
        self.append::<u32>(data.len() as u32);
        for v in data.iter() {
            self.append::<T>(v.clone());
        }
        data.len() * size_of::<T>() + 4
    }

    pub fn buf(&self) -> Vec<u8> {
        self.buf.clone()
    }
}
