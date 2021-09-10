use byteorder::{LittleEndian, ReadBytesExt};
use indexmap::IndexMap;
use std::{
    cell::RefCell,
    collections::HashMap,
    convert::TryInto,
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::Path,
    rc::Rc,
};

pub struct WadFile {
    file: RefCell<BufReader<File>>,
    wad_type: WadType,
    lump_locations: Vec<Lump>,
    lump_indices: HashMap<String, LumpIndex>,
    lump_cache: RefCell<Vec<Option<Rc<[u8]>>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LumpIndex {
    Unique(usize),
    NotUnique,
}

impl WadFile {
    pub fn open(path: impl AsRef<Path>) -> io::Result<WadFile> {
        let file = File::open(path)?;
        let mut file = BufReader::new(file);

        let header = Header::read_from(&mut file)?;

        let directory =
            Directory::read_from(&mut file, header.directory_offset, header.lump_count)?;
        let lump_locations = directory.lumps;

        let mut lump_indices = HashMap::new();
        for (index, lump) in lump_locations.iter().enumerate() {
            lump_indices
                .entry(lump.name.clone())
                .and_modify(|e| *e = LumpIndex::NotUnique)
                .or_insert(LumpIndex::Unique(index));
        }

        let lump_cache = vec![None; lump_locations.len()];

        Ok(WadFile {
            wad_type: header.wad_type,
            file: RefCell::new(file),
            lump_locations,
            lump_indices,
            lump_cache: RefCell::new(lump_cache),
        })
    }

    pub fn lump(&self, name: &str) -> io::Result<Rc<[u8]>> {
        let index = self.lump_index(name)?;

        let mut lump_cache = self.lump_cache.borrow_mut();
        let cached_contents = &mut lump_cache[index];

        if cached_contents.is_none() {
            // Get lump metadata.
            let index = self.lump_index(name)?;
            let lump = &self.lump_locations[index];

            // Read lump from file.
            let mut file = self.file.borrow_mut();
            let mut contents = vec![0u8; lump.size.try_into().unwrap()];
            file.seek(SeekFrom::Start(lump.offset.into()))?;
            file.read_exact(&mut contents)?;

            // Add to cache.
            let contents: Rc<[u8]> = contents.into_boxed_slice().into();
            *cached_contents = Some(contents.clone());
        }

        Ok(cached_contents.as_ref().unwrap().clone())
    }

    pub fn lumps_between(&self, start: &str, end: &str) -> io::Result<IndexMap<String, Rc<[u8]>>> {
        let start_index = self.lump_index(start)?;
        let end_index = self.lump_index(end)?;

        let mut lumps = IndexMap::with_capacity(end_index - start_index - 2);
        for index in (start_index + 1)..(end_index - 1) {
            let name = &self.lump_locations[index].name;
            lumps.insert(name.clone(), self.lump(name)?);
        }
        Ok(lumps)
    }

    fn lump_index(&self, name: &str) -> io::Result<usize> {
        let index = self.lump_indices.get(name).copied().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("no lump named {}", name))
        })?;

        match index {
            LumpIndex::Unique(index) => Ok(index),
            LumpIndex::NotUnique => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} not unique", name),
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

        let mut lumps = Vec::with_capacity(lump_count.try_into().unwrap());

        for _ in 0..lump_count {
            let offset = file.read_u32::<LittleEndian>()?;
            let size = file.read_u32::<LittleEndian>()?;
            let mut name = [0u8; 8];
            file.read_exact(&mut name)?;
            let name = std::str::from_utf8(&name)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .trim_end_matches('\0')
                .to_owned();

            lumps.push(Lump { offset, size, name });
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
        assert_eq!(wad.lump_locations.len(), 1264);

        let wad = test_wad("killer.wad")?;
        assert_eq!(wad.wad_type, WadType::Patch);
        assert_eq!(wad.lump_locations.len(), 55);

        Ok(())
    }

    #[test]
    fn read_lumps() -> io::Result<()> {
        let wad = test_wad("doom.wad")?;

        assert_eq!(wad.lump("DEMO1")?.len(), 20118);
        assert_eq!(wad.lump("E1M1")?.len(), 0);

        // Test cache.
        assert!(wad.lump_cache.borrow()[wad.lump_index("PNAMES")?].is_none());
        assert_eq!(wad.lump("PNAMES")?.len(), 2804);
        assert!(wad.lump_cache.borrow()[wad.lump_index("PNAMES")?].is_some());
        assert_eq!(wad.lump("PNAMES")?.len(), 2804);

        Ok(())
    }

    #[test]
    fn detect_duplicates() -> io::Result<()> {
        let wad = test_wad("doom.wad")?;

        assert!(wad.lump_index("E1M1").is_ok());
        assert!(wad.lump_index("THINGS").is_err());
        assert!(wad.lump_index("VERTEXES").is_err());
        assert!(wad.lump_index("SECTORS").is_err());
        
        Ok(())
    }

    #[test]
    fn lumps_between() -> io::Result<()> {
        let wad = test_wad("doom.wad")?;

        let sprites = wad.lumps_between("S_START", "S_END")?;
        assert_eq!(sprites.len(), 482);
        assert_eq!(sprites.get_index(100).unwrap().0, "SARGB5");

        Ok(())
    }

    fn test_wad(path: impl AsRef<Path>) -> io::Result<WadFile> {
        WadFile::open(Path::new("test").join(path))
    }
}
