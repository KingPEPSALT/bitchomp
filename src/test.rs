use std::time::{Duration, Instant};

use crate::ChompFlatten;

use super::{ByteError, ByteReader, ByteWriter, Endianness};

#[test]
fn test_bytewriter() -> Result<(), ByteError> {
    let mut writer = ByteWriter::new(Endianness::default());
    writer.append::<u16>(10);
    writer.append::<String>(String::from("testing"));
    writer.append::<i32>(-14);

    let b = writer.buf();
    let mut reader = ByteReader::new(b.as_slice(), Endianness::default());
    assert_eq!(reader.read::<u16>()?.inner(), 10);
    assert_eq!(reader.read_string()?, String::from("testing"));
    assert_eq!(reader.read::<i32>()?.inner(), -14);
    Ok(())
}

#[test]
fn test_bytereader() -> Result<(), ByteError> {
    let data = std::fs::read("test/texture.text")?;
    let mut reader = ByteReader::new(&data, Endianness::default());
    assert_eq!(reader.read::<u16>()?.inner(), 1);
    assert_eq!(reader.read::<u16>()?.inner(), 0);
    assert_eq!(reader.read::<u32>()?.inner(), 0x4000);
    assert_eq!(reader.read::<u32>()?.inner(), 0x2B0C);
    assert_eq!(reader.read::<u32>()?.inner(), 0);
    assert_eq!(reader.read::<u16>()?.inner(), 0x0080);
    assert_eq!(reader.read::<u16>()?.inner(), 0x0080);
    assert_eq!(reader.read::<u16>()?.inner(), 0x0049);
    reader.seek(0x1c)?;
    assert_eq!(
        reader.read_n::<u32>(14)?.flatten(),
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
    println!("reader.read_n::<u16>({})?: {:#?}", len / 2, timer.time());

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
    let data = vec![
        1, 1, 1, 12, 0, 0, 0, 0, 16, 1, 0, 0, 0, 85, 85, 21, 0, 1, 16, 1, 0, 0, 0, 85, 85, 21, 0,
        2, 16, 1, 0, 0, 0, 85, 85, 21, 0, 3, 16, 1, 0, 0, 0, 85, 85, 21, 0, 4, 16, 1, 0, 0, 0, 85,
        85,
    ];
    let mut reader = ByteReader::new(&data, Endianness::Little);
    println!(
        "{:x?}",
        reader
            .peek_n::<u8>(3)?
            .iter()
            .map(|v| v.0)
            .collect::<Vec<*const u8>>()
    );
    assert_eq!(reader.read_n::<u8>(3)?.flatten(), vec![1, 1, 1]);

    println!("{:x?}", reader.peek_n::<u32>(1)?[0].0);
    assert_eq!(reader.peek::<u32>()?.inner(), 12);
    let vec: Vec<u32> = reader.read_sized_vector::<u32>()?.flatten();
    assert_eq!(
        vec,
        vec![
            69632, 1431633920, 268501013, 1, 1398101, 69634, 1431633920, 268632085, 1, 1398101,
            69636, 1431633920
        ]
    );
    Ok(())
}
