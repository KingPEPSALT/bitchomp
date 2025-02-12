//! bytereader.rs
use core::fmt;
use std::{
    cmp,
    convert::Infallible,
    fmt::Debug,
    io::{self, BufRead, Write},
    marker::PhantomData,
    mem::size_of,
};
use thiserror::Error;

use crate::Chomp;

use super::{Endianness, TryFromBytes, TryFromBytesError};

// resource should have the same lifetime as the bytes.
pub trait ByteReaderResource<'a> =
    TryFromBytes<Bytes: From<&'a [u8]>, Error = TryFromBytesError> + Clone;
/// Error returned by ByteReader
#[derive(Error)]
pub struct ByteReaderError {
    kind: ByteReaderErrorKind,
    cursor: usize,
}

impl std::fmt::Debug for ByteReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ByteReaderError")
            .field("kind", &self.kind)
            .field_with("cursor", |f| write!(f, "{:#x}", &self.cursor))
            .finish()
    }
}

impl std::fmt::Display for ByteReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ByteReaderError")
            .field("kind", &self.kind)
            .field_with("cursor", |f| write!(f, "{:#x}", &self.cursor))
            .finish()
    }
}

#[derive(Debug, Error)]
pub enum ByteReaderErrorKind {
    #[error("no bytes")]
    NoBytes,

    #[error("try from bytes error: {0}")]
    TryFromBytesError(TryFromBytesError),

    #[error("I/O error: {0}")]
    IOError(io::Error),

    #[error("infallible")]
    Infallible,
}

impl From<std::io::Error> for ByteReaderErrorKind {
    fn from(err: std::io::Error) -> Self {
        ByteReaderErrorKind::IOError(err)
    }
}

impl From<TryFromBytesError> for ByteReaderErrorKind {
    fn from(err: TryFromBytesError) -> Self {
        ByteReaderErrorKind::TryFromBytesError(err)
    }
}

impl From<Infallible> for ByteReaderErrorKind {
    fn from(_: Infallible) -> Self {
        ByteReaderErrorKind::Infallible
    }
}

impl From<Infallible> for ByteReaderError {
    fn from(_: Infallible) -> Self {
        ByteReaderError {
            kind: ByteReaderErrorKind::Infallible,
            cursor: 20, // OK ??
        }
    }
}

/// A tool for reading bytes from a buffer
#[derive(Clone)]
pub struct ByteReader<'a> {
    /// A reference to the data that is being read.
    buf: &'a [u8],
    /// The current place in the buffer.
    pub cursor: &'a [u8],
    /// The endianness in which the bytes should be read as.
    endianness: Endianness,
}

impl<'a> ByteReader<'a> {
    /// Returns a ByteReaderError with the context of the ByteReader
    ///
    /// # Arguments
    ///
    /// * `kind` - the kind of error to receive
    pub fn err(&self, kind: ByteReaderErrorKind) -> ByteReaderError {
        ByteReaderError {
            kind,
            cursor: self.cursor(),
        }
    }

    /// Returns a ByteReader reading from buf
    ///
    /// # Arguments
    ///
    /// * `buf` - A slice of u8, the buffer of bytes
    /// * `endianness` - The endianness in which the bytes should be read
    ///
    /// # Examples
    /// ```
    /// #![feature(generic_const_exprs)]
    ///
    /// use bitchomp::{ByteReader, Endianness, ByteError};
    ///
    /// fn main() -> Result<(), ByteError> {
    ///     // get buffer from file
    ///     let buf = std::fs::read("test/binary.file")?;
    ///     let mut reader = ByteReader::new(&buf, Endianness::default());
    ///     // alternatively:
    ///     //  let mut reader = ByteReader::new(buf, Endianness::Big));
    ///     
    ///     // ... do stuff    
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(buf: &'a [u8], endianness: Endianness) -> Self {
        ByteReader {
            buf,
            cursor: buf,
            endianness,
        }
    }

    /// Returns a ByteReaderIterator<T> that iterates over a buffer, returing bytes of type T
    ///
    /// # Examples
    /// ```
    /// #![feature(generic_const_exprs)]
    ///
    /// use bitchomp::{ByteReader, Endianness, ByteError};
    /// // get buffer from file
    /// fn main() -> Result<(), ByteError> {
    ///     let buf = std::fs::read("test/values.file")?;
    ///     let mut reader = ByteReader::new(&buf, Endianness::Little);
    ///     let values_squared: Vec<u16> = reader.iter::<u16>().map(|t|t*t).collect();
    ///     
    ///     // ... do stuff    
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn iter<T: TryFromBytes>(&'a mut self) -> ByteReaderIterator<T>
    where
        T::Bytes: TryFrom<Vec<u8>>,
    {
        ByteReaderIterator::<T> {
            buf: self,
            resource_type: PhantomData,
        }
    }

    /// Returns the length of the remaining buffer
    pub fn len(&self) -> usize {
        self.cursor.len()
    }

    /// Returns the cursor position
    pub fn cursor(&self) -> usize {
        self.buf.len() - self.cursor.len()
    }

    /// Seeks to a position in the buffer
    ///
    /// # Arguments
    ///
    /// * `pos` - the position in the buffer
    ///
    /// # Examples
    /// ```
    /// #![feature(generic_const_exprs)]
    ///
    /// use bitchomp::{ByteError, ByteReader, Endianness};
    ///
    /// fn main() -> Result<(), ByteError> {
    ///     let buf = std::fs::read("test/binary.file")?;
    ///     let mut reader = ByteReader::new(&buf, Endianness::Little);
    ///     if reader.seek(15).is_ok() {
    ///         let value = reader.read::<u32>()?.inner();
    ///     }
    ///
    ///     // ... do stuff
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn seek(&mut self, pos: usize) -> Result<(), ByteReaderError> {
        if pos > self.buf.len() {
            return Err(self.err(ByteReaderErrorKind::NoBytes));
        }
        self.cursor = &self.buf[pos..];
        Ok(())
    }

    /// Reads a type T from the buffer
    ///
    /// # Arguments
    ///
    /// * `T: FromBytes` - the type you want to read
    ///
    /// # Examples
    /// ```
    /// #![feature(generic_const_exprs)]
    ///
    /// use bitchomp::{ByteError, ByteReader, Endianness};
    ///
    /// fn main() -> Result<(), ByteError>{
    ///     // get buffer
    ///     let buf = std::fs::read("test/binary.file")?;
    ///     let mut reader = ByteReader::new(&buf, Endianness::Little);
    ///
    ///     let value = reader.read::<u32>()?.inner();
    ///     // only reads 2 bytes!
    ///     let next_value = reader.read::<u16>()?.inner() as u32;
    ///     
    ///     // do stuff...
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn read<T: ByteReaderResource<'a>>(&mut self) -> Result<Chomp<T>, ByteReaderError> {
        Ok(self.read_n(1)?[0].clone())
    }

    /// Reads a type T from the buffer
    ///
    /// # Arguments
    ///
    /// * `T: FromBytes` - the type you want to read
    ///
    /// # Examples
    /// ```
    /// #![feature(generic_const_exprs)]
    ///
    /// use bitchomp::{ByteError, ByteReader, Endianness};
    ///
    /// fn main() -> Result<(), ByteError>{
    ///     // get buffer
    ///     let buf = std::fs::read("test/binary.file")?;
    ///         
    ///     let mut reader = ByteReader::new(&buf, Endianness::Little);
    ///     
    ///     // doesn't consume the next 4 bytes!
    ///     let full_value = reader.peek::<u32>()?.inner();
    ///
    ///     // does consume the next 4 bytes!
    ///     let [first_half, second_half] = [reader.read::<u16>()?.inner() as u32, reader.read::<u16>()?.inner() as u32];
    ///     
    ///     // do stuff...
    ///  
    ///     Ok(())
    /// }
    /// ```
    pub fn peek<T: ByteReaderResource<'a>>(&mut self) -> Result<Chomp<T>, ByteReaderError> {
        Ok(self.peek_n(1)?[0].clone())
    }

    /// Gets the size of the buffer
    pub fn size(&self) -> usize {
        self.buf.len()
    }

    // std::mem::size_of<T> for T: ByteReaderResource + Size
    pub fn byte_size<T: ByteReaderResource<'a>>(&mut self) -> Result<usize, ByteReaderError> {
        let (_, s) = T::try_from_bytes(self.cursor.into(), self.endianness)
            .map_err(|e| self.err(ByteReaderErrorKind::TryFromBytesError(e)))?;
        Ok(s)
    }

    /// Reads a type T from the buffer
    ///
    /// # Arguments
    ///
    /// * `T: FromBytes` - the type you want to read
    /// * `n: usize` - the number of T to read
    /// # Examples
    /// ```
    /// #![feature(generic_const_exprs)]
    ///
    /// use bitchomp::{ByteError, ByteReader, Endianness, ChompFlatten};
    ///
    /// fn main() -> Result<(), ByteError> {
    ///
    ///     // get buffer
    ///     let buf = std::fs::read("test/binary.file")?;
    ///     let mut reader = ByteReader::new(&buf, Endianness::Little);
    ///
    ///     // read 32 bytes (or 16 u16s)
    ///     let values = reader.read_n::<u16>(16)?.flatten();
    ///     
    ///     // do stuff ...
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn read_n<T: ByteReaderResource<'a> + Sized>(
        &mut self,
        n: usize,
    ) -> Result<Vec<Chomp<T>>, ByteReaderError> {
        // handle the error here to avoid consuming bytes we don't have
        let res = self.peek_n::<T>(n)?;
        self.consume(n * std::mem::size_of::<T>());
        Ok(res)
    }
    /// Reads a type T from the buffer n times without consuming
    ///
    /// # Arguments
    ///
    /// * `T: FromBytes` - the type you want to read
    /// * `n` - the number of T to read
    ///
    /// # Examples
    /// ```
    /// #![feature(generic_const_exprs)]
    ///
    /// use bitchomp::{ByteReader, ByteError, Endianness, ChompFlatten};
    ///
    /// fn main() -> Result<(), ByteError> {
    ///     // get buffer
    ///     let buf = std::fs::read("test/binary.file")?;
    ///     let mut reader = ByteReader::new(&buf, Endianness::Little);
    ///
    ///     // read 32 bytes (or 16 u16s)
    ///     let values = reader.read_n::<u16>(16)?.flatten();
    ///
    ///     // ... do stuff
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn peek_n<T: ByteReaderResource<'a> + Sized>(
        &self,
        n: usize,
    ) -> Result<Vec<Chomp<T>>, ByteReaderError> {
        if self.len() / size_of::<T>() < n {
            return Err(ByteReaderError {
                kind: ByteReaderErrorKind::NoBytes,
                cursor: self.cursor(),
            });
        }
        Ok(unsafe {
            std::mem::transmute::<&[u8], &[T]>(&self.cursor)
                .iter()
                .map(|t| Chomp::new(t))
                .take(n)
                .collect::<Vec<Chomp<T>>>()
        })
    }

    pub fn read_string(&mut self) -> Result<String, ByteReaderError> {
        let (value, size) = String::try_from_bytes(self.cursor.into(), self.endianness)
            .map_err(|e| self.err(ByteReaderErrorKind::TryFromBytesError(e)))?;
        self.consume(size);
        Ok(value)
    }

    /// Reads a type T from the buffer n times without consuming
    ///
    /// # Arguments
    ///
    /// * `T: FromBytes` - the type you want to read
    /// * `n` - the number of T to read
    ///
    /// # Examples
    /// ```
    /// #![feature(generic_const_exprs)]
    ///
    /// use bitchomp::{ByteReader, ByteError, Endianness, ChompFlatten};
    ///
    /// fn main() -> Result<(), ByteError> {
    ///     // get buffer
    ///     let buf = std::fs::read("test/binary.file")?;
    ///     let mut reader = ByteReader::new(&buf, Endianness::Little);
    ///
    ///     // read 32 bytes (or 16 u16s)
    ///     let values = reader.read_n::<u16>(16)?.flatten();
    ///
    ///     // ... do stuff
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn read_sized_vector<T: ByteReaderResource<'a> + Copy>(
        &mut self,
    ) -> Result<Vec<Chomp<T>>, ByteReaderError> {
        let size = self.read::<u32>()?.inner() as usize;
        self.read_n::<T>(size)
    }

    pub fn read_remaining<T: ByteReaderResource<'a> + Copy>(
        &mut self,
    ) -> Result<Vec<Chomp<T>>, ByteReaderError> {
        self.read_n::<T>(self.len() / size_of::<T>())
    }

    pub fn rebase(&mut self, pos: usize) {
        self.buf = &self.buf[pos..];
        self.cursor = self.buf;
    }
}

impl<'a> io::Read for ByteReader<'a> {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        buf.write(self.buf)?;
        // probs incorrect
        Ok(cmp::min(self.cursor.len(), buf.len()))
    }
}

impl<'a> BufRead for ByteReader<'a> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Ok(self.cursor)
    }

    fn consume(&mut self, amt: usize) {
        self.cursor = &self.cursor[amt..];
    }
}

pub struct ByteReaderIterator<'a, T: TryFromBytes> {
    buf: &'a mut ByteReader<'a>,
    resource_type: PhantomData<T>,
}

impl<'a, T: ByteReaderResource<'a> + fmt::Debug> Iterator for ByteReaderIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.read::<T>().map(|v| v.inner()).ok()
    }
}
