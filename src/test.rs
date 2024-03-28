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
    let data = std::fs::read("texture.text")?;
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
