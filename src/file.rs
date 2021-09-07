use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read, Seek, SeekFrom},
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
struct WadHeader {
    pub wad_type: WadType,
    pub lump_count: u32,
    pub directory_offset: u32,
}

impl WadHeader {
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
    wad_type: WadType,
    directory_offset: u32,
    lumps: Vec<Lump>,
    lumps_by_name: BTreeMap<String, u32>,
}

impl WadFile {
    pub fn open(path: impl AsRef<Path>) -> io::Result<WadFile> {
        Self::read_from(&mut File::open(path)?)
    }

    fn read_from(file: impl Read + Seek) -> io::Result<WadFile> {
        let mut wad_file = WadFile {
            wad_type: WadType::Initial,
            directory_offset: 0,
            lumps: Vec::new(),
            lumps_by_name: BTreeMap::new(),
        };

        let header = WadHeader::read_from(file)?;
        wad_file.wad_type = header.wad_type;
        wad_file.directory_offset = header.directory_offset;

        Ok(wad_file)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iwad_vs_pwad() -> io::Result<()> {
        let iwad = test_wad("doom.wad")?;
        assert_eq!(iwad.wad_type, WadType::Initial);

        let pwad = test_wad("killer.wad")?;
        assert_eq!(pwad.wad_type, WadType::Patch);

        Ok(())
    }

    fn test_wad(path: impl AsRef<Path>) -> io::Result<WadFile> {
        WadFile::open(Path::new("test").join(path))
    }
}
