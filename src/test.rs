use std::time::{Duration, Instant};

use super::{ByteError, ByteReader, ByteWriter, Endianness};

#[test]
fn test_bytewriter() -> Result<(), ByteError> {
    let mut writer = ByteWriter::new(Endianness::default());
    writer.append::<u16>(10);
    writer.append::<String>(String::from("testing"));
    writer.append::<i32>(-14);

    let b = writer.buf();
    let mut reader = ByteReader::new(b.as_slice(), Endianness::default());
    assert_eq!(reader.read::<u16>()?, 10);
    assert_eq!(reader.read::<String>()?, String::from("testing"));
    assert_eq!(reader.read::<i32>()?, -14);
    Ok(())
}

#[test]
fn test_bytereader() -> Result<(), ByteError> {
    let data = std::fs::read("test/texture.text")?;
    let mut reader = ByteReader::new(&data, Endianness::default());
    assert_eq!(reader.read::<u16>()?, 1);
    assert_eq!(reader.read::<u16>()?, 0);
    assert_eq!(reader.read::<u32>()?, 0x4000);
    assert_eq!(reader.read::<u32>()?, 0x2B0C);
    assert_eq!(reader.read::<u32>()?, 0);
    assert_eq!(reader.read::<u16>()?, 0x0080);
    assert_eq!(reader.read::<u16>()?, 0x0080);
    assert_eq!(reader.read::<u16>()?, 0x0049);
    reader.seek(0x1c)?;
    assert_eq!(
        reader.read_n::<u32>(14)?,
        vec![8192, 10240, 10752, 10880, 10912, 10920, 10928, 10936, 0, 0, 0, 0, 0, 0,]
    );
    Ok(())
}

/// Utility class for timing functions
struct Timer {
    instant: Instant,
}

impl Timer {
    fn new() -> Self {
        Self {
            instant: Instant::now(),
        }
    }

    fn restart(&mut self) {
        self.instant = Instant::now();
    }
    fn time(&mut self) -> Duration {
        let ret = self.instant.elapsed();
        self.restart();
        return ret;
    }
}

#[test]
fn benchmark_bytereader() -> Result<(), ByteError> {
    let mut timer = Timer::new();
    let data = std::fs::read("test/texture.text")?;
    println!("std::fs::read(\"texture.text\"): {:#?}", timer.time());
    let mut reader = ByteReader::new(&data, Endianness::default());
    println!(
        "ByteReader::new(&data, Endianness::default()): {:#?}",
        timer.time()
    );
    let mut bytes = reader.read_remaining::<u8>()?;
    let len = bytes.len();
    println!(
        "reader.read_remaining::<u8>()?: {:#?} - {:#?} bytes",
        timer.time(),
        len
    );
    reader.rebase(0);
    bytes = reader.read_n::<u8>(len)?;
    println!("reader.read_n::<u8>({})?: {:#?}", len, timer.time());
    assert_eq!(bytes.len(), len);
    reader.rebase(0);
    
    timer.restart();
    // assign to avoid it getting thrown out
    let _new_bytes = reader.read_n::<u16>(len / 2)?;
    assert_eq!(bytes.len(), len);
    println!(
        "reader.read_n::<u16>({})?: {:#?}",
        len / 2,
        timer.time()
    );

    Ok(())
}

#[test]
fn bytereader_raw_reading() -> Result<(), ByteError> {
    let data = std::fs::read("test/texture.text")?;
    let mut reader = ByteReader::new(&data, Endianness::default());
    reader.read_n::<u16>(data.len() / 2)?;
    reader.rebase(0);
    assert!(reader.read_n::<u16>(data.len()).is_err());
    Ok(())
}

#[test]
fn test_bytereader_read_sized_vector() -> Result<(), ByteError> {
    let data = vec![0, 0, 0, 3, 250, 230, 210];
    let mut reader = ByteReader::new(&data, Endianness::Big);
    assert_eq!(reader.peek::<u32>()?, 3);
    assert_eq!(reader.read_sized_vector::<u8>()?, vec![250, 230, 210]);
    Ok(())
    
}