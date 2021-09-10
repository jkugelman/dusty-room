use byteorder::{LittleEndian, ReadBytesExt};
use std::{cell::RefCell, collections::HashMap, convert::TryInto, fs::File, io::{self, BufReader, Read, Seek, SeekFrom}, path::Path, rc::Rc};

pub struct WadFile {
    wad_type: WadType,
    lump_cache: RefCell<HashMap<String, Rc<[u8]>>>,

    file: RefCell<BufReader<File>>,
    lumps_by_index: Vec<Lump>,
    lumps_by_name: HashMap<String, usize>,
}

impl WadFile {
    pub fn open(path: impl AsRef<Path>) -> io::Result<WadFile> {
        let file = File::open(path)?;
        let mut file = BufReader::new(file);

        let header = Header::read_from(&mut file)?;

        let directory =
            Directory::read_from(&mut file, header.directory_offset, header.lump_count)?;
        let lump_vec = directory.lumps;
        let mut lump_map = HashMap::new();
        for (i, lump) in lump_vec.iter().enumerate() {
            lump_map.insert(lump.name.clone(), i);
        }

        Ok(WadFile {
            wad_type: header.wad_type,
            lump_cache: RefCell::new(HashMap::new()),

            file: RefCell::new(file),
            lumps_by_index: lump_vec,
            lumps_by_name: lump_map,
        })
    }

    pub fn lump(&self, name: &str) -> io::Result<Rc<[u8]>> {
        let mut lump_cache = self.lump_cache.borrow_mut();

        // Already in cache?
        if let Some(contents) = lump_cache.get(name) {
            return Ok(contents.clone());
        }

        // Get lump metadata.
        let index = *self.lumps_by_name.get(name).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("no lump named {}", name))
        })?;
        let lump = self.lumps_by_index.get(index).unwrap();

        // Read lump from file.
        let mut file = self.file.borrow_mut();
        let mut contents = vec![0u8; lump.size.try_into().unwrap()];
        file.seek(SeekFrom::Start(lump.offset.into()))?;
        file.read_exact(&mut contents)?;

        // Add to cache.
        let contents: Rc<[u8]> = contents.into_boxed_slice().into();
        lump_cache.insert(name.to_owned(), contents.clone());

        Ok(contents)
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
struct Directory {
    lumps: Vec<Lump>,
}

impl Directory {
    fn read_from(
        mut file: impl Read + Seek,
        directory_offset: u32,
        lump_count: u32,
    ) -> io::Result<Self> {
        file.seek(SeekFrom::Start(directory_offset.into()))?;

        let mut lumps = Vec::new();

        for _ in 0..lump_count {
            let offset = file.read_u32::<LittleEndian>()?;
            let size = file.read_u32::<LittleEndian>()?;
            let mut name = [0u8; 8];
            file.read_exact(&mut name)?;
            let name = std::str::from_utf8(&name)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .trim_end_matches('\0')
                .to_owned();

            lumps.push(Lump { offset, size, name })
        }

        Ok(Directory { lumps })
    }
}

#[derive(Debug)]
struct Lump {
    offset: u32,
    size: u32,
    name: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn header() -> io::Result<()> {
        let wad = test_wad("doom.wad")?;
        assert_eq!(wad.wad_type, WadType::Initial);
        assert_eq!(wad.lumps_by_index.len(), 1264);
        assert_eq!(wad.lumps_by_index[0].name, "PLAYPAL");
        assert_eq!(wad.lumps_by_index[1].name, "COLORMAP");
        assert_eq!(wad.lumps_by_index[2].name, "ENDOOM");
        assert_eq!(wad.lumps_by_index[3].name, "DEMO1");
    
        let wad = test_wad("killer.wad")?;
        assert_eq!(wad.wad_type, WadType::Patch);
        assert_eq!(wad.lumps_by_index.len(), 55);
        assert_eq!(wad.lumps_by_index[0].name, "E1M1");
        assert_eq!(wad.lumps_by_index[1].name, "THINGS");
        assert_eq!(wad.lumps_by_index[2].name, "LINEDEFS");
        assert_eq!(wad.lumps_by_index[3].name, "SIDEDEFS");

        Ok(())
    }

    #[test]
    fn read_lumps() -> io::Result<()> {
        let wad = test_wad("doom.wad")?;

        assert_eq!(wad.lump("DEMO1")?.len(), 20118);
        assert_eq!(wad.lump("E1M1")?.len(), 0);

        // Test cache.
        assert!(!wad.lump_cache.borrow().contains_key("PNAMES"));
        assert_eq!(wad.lump("PNAMES")?.len(), 2804);
        assert!(wad.lump_cache.borrow().contains_key("PNAMES"));
        assert_eq!(wad.lump("PNAMES")?.len(), 2804);

        Ok(())
    }

    fn test_wad(path: impl AsRef<Path>) -> io::Result<WadFile> {
        WadFile::open(Path::new("test").join(path))
    }
}
