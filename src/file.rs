use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::Path,
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum WadType {
    Initial,
    Patch,
}

impl WadType {
    fn read_from(mut file: impl Read) -> io::Result<Self> {
        let mut buffer = [0u8; 4];
        file.read_exact(&mut buffer)?;

        match &buffer {
            b"IWAD" => Ok(WadType::Initial),
            b"PWAD" => Ok(WadType::Patch),

            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{:?} neither IWAD nor PWAD", buffer),
            )),
        }
    }
}

#[derive(Debug)]
struct Header {
    pub wad_type: WadType,
    pub lump_count: u32,
    pub directory_offset: u32,
}

impl Header {
    fn read_from(mut file: impl Read + Seek) -> io::Result<Self> {
        file.seek(SeekFrom::Start(0))?;

        let wad_type = WadType::read_from(&mut file)?;
        let lump_count = file.read_u32::<LittleEndian>()?;
        let directory_offset = file.read_u32::<LittleEndian>()?;

        Ok(Self {
            wad_type,
            lump_count,
            directory_offset,
        })
    }
}

#[derive(Debug)]
struct Lump {
    pub index: u32,
    pub offset: u32,
    pub size: u32,
    pub name: String,
}

pub struct WadFile {
    header: Header,
}

impl WadFile {
    pub fn open(path: impl AsRef<Path>) -> io::Result<WadFile> {
        let file = File::open(path)?;
        let mut file = BufReader::new(file);

        Self::read_from(&mut file)
    }

    fn read_from(file: impl Read + Seek) -> io::Result<WadFile> {
        let header = Header::read_from(file)?;
        Ok(WadFile { header })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn header() -> io::Result<()> {
        let header = test_wad("doom.wad")?.header;
        assert_eq!(header.wad_type, WadType::Initial);
        assert_eq!(header.lump_count, 1264);
        assert_eq!(header.directory_offset, 0x3fb7b4);

        let header = test_wad("killer.wad")?.header;
        assert_eq!(header.wad_type, WadType::Patch);
        assert_eq!(header.lump_count, 55);
        assert_eq!(header.directory_offset, 0x90508);

        Ok(())
    }

    fn test_wad(path: impl AsRef<Path>) -> io::Result<WadFile> {
        WadFile::open(Path::new("test").join(path))
    }
}
