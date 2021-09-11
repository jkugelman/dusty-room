use byteorder::{LittleEndian, ReadBytesExt};
use indexmap::IndexMap;
use std::{
    cell::RefCell,
    collections::HashMap,
    convert::TryInto,
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Arc,
};

/// A single IWAD or PWAD file.
pub struct WadFile {
    path: PathBuf,
    file: RefCell<BufReader<File>>,
    wad_type: WadType,
    lump_locations: Vec<LumpLocation>,
    lump_indices: HashMap<String, LumpIndex>,
    lump_cache: RefCell<Vec<Option<Arc<[u8]>>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LumpIndex {
    Unique(usize),
    NotUnique,
}

pub type LumpBlock = IndexMap<String, Arc<[u8]>>;

impl WadFile {
    /// Reads a WAD file from disk.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let mut file = BufReader::new(file);

        let header = Header::read_from(&mut file)?;

        let directory =
            Directory::read_from(&mut file, header.directory_offset, header.lump_count)?;
        let lump_locations = directory.lumps;

        let mut lump_indices = HashMap::new();
        for (index, lump_location) in lump_locations.iter().enumerate() {
            lump_indices
                .entry(lump_location.name.clone())
                .and_modify(|e| *e = LumpIndex::NotUnique)
                .or_insert(LumpIndex::Unique(index));
        }

        let lump_cache = vec![None; lump_locations.len()];

        Ok(WadFile {
            path: path.to_owned(),
            wad_type: header.wad_type,
            file: RefCell::new(file),
            lump_locations,
            lump_indices,
            lump_cache: RefCell::new(lump_cache),
        })
    }

    /// File path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Is this an IWAD or PWAD?
    pub fn wad_type(&self) -> WadType {
        self.wad_type
    }

    /// Retrieves a named lump. The name must be unique.
    pub fn lump(&self, name: &str) -> io::Result<Arc<[u8]>> {
        let index = self.lump_index(name)?;
        self.lump_contents(index)
    }

    /// Retrieves a block of `size` lumps following a named marker. The marker lump
    /// is not included in the result.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use kdoom::WadFile;
    ///
    /// let wad = WadFile::open("doom.wad")?;
    /// let map = wad.lumps_after("E1M5", 10)?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn lumps_after(&self, start: &str, size: usize) -> io::Result<LumpBlock> {
        let start_index = self.lump_index(start)? + 1;

        if start_index + size >= self.lump_locations.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} does not have {} lumps after", start, size),
            ));
        }

        self.lump_block(start_index, size)
    }

    /// Retrieves a block of lumps between start and end markers. The marker lumps
    /// are not included in the result.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use kdoom::WadFile;
    ///
    /// let wad = WadFile::open("doom.wad")?;
    /// let sprites = wad.lumps_between("S_START", "S_END")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn lumps_between(&self, start: &str, end: &str) -> io::Result<LumpBlock> {
        let start_index = self.lump_index(start)? + 1;
        let end_index = self.lump_index(end)?;

        if start_index >= end_index {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} after {}", start, end),
            ));
        }

        let size = end_index - start_index;
        self.lump_block(start_index, size)
    }

    /// Looks up a lump's index.
    ///
    /// # Errors
    ///
    /// * `io::ErrorKind::NotFound` if the lump isn't found.
    /// * `io::ErrorKind::InvalidInput` if the lump name isn't unique.
    fn lump_index(&self, name: &str) -> io::Result<usize> {
        let index = self.lump_indices.get(name).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("no lump named {}", name))
        })?;

        match *index {
            LumpIndex::Unique(index) => Ok(index),
            LumpIndex::NotUnique => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} not unique", name),
            )),
        }
    }

    /// Loads a lump from disk and caches it to speed up future lookups.
    fn lump_contents(&self, index: usize) -> io::Result<Arc<[u8]>> {
        let mut lump_cache = self.lump_cache.borrow_mut();
        let cached_contents = &mut lump_cache[index];

        if cached_contents.is_none() {
            let lump_location = &self.lump_locations[index];

            let contents: Arc<[u8]> = if lump_location.size > 0 {
                // Read lump from file.
                let mut file = self.file.borrow_mut();
                let mut contents = vec![0u8; lump_location.size.try_into().unwrap()];
                file.seek(SeekFrom::Start(lump_location.offset.into()))?;
                file.read_exact(&mut contents)?;
                contents.into_boxed_slice().into()
            } else {
                // Empty lump. The offset may be garbage; avoid seeking to it.
                Arc::new([0u8; 0])
            };

            // Add to cache.
            *cached_contents = Some(contents.clone());
        }

        Ok(cached_contents.as_ref().unwrap().clone())
    }

    /// Retrieves a block of lumps.
    fn lump_block(&self, start_index: usize, size: usize) -> io::Result<LumpBlock> {
        let mut lumps = IndexMap::with_capacity(size);

        for index in start_index..start_index + size {
            let name = &self.lump_locations[index].name;
            lumps.insert(name.clone(), self.lump_contents(index)?);
        }

        Ok(lumps)
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
pub enum WadType {
    /// IWAD
    Initial,
    /// PWAD
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
    lumps: Vec<LumpLocation>,
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

            lumps.push(LumpLocation { offset, size, name });
        }

        Ok(Directory { lumps })
    }
}

#[derive(Debug)]
struct LumpLocation {
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
    fn lumps_after() -> io::Result<()> {
        let wad = test_wad("doom.wad")?;

        let map = wad.lumps_after("E1M8", 10)?;
        assert_eq!(map.len(), 10);
        assert_eq!(
            map.keys().collect::<Vec<_>>(),
            [
                "THINGS", "LINEDEFS", "SIDEDEFS", "VERTEXES", "SEGS", "SSECTORS", "NODES",
                "SECTORS", "REJECT", "BLOCKMAP"
            ],
        );

        Ok(())
    }

    #[test]
    fn lumps_between() -> io::Result<()> {
        let wad = test_wad("doom.wad")?;

        let sprites = wad.lumps_between("S_START", "S_END")?;
        assert_ne!(sprites.first().unwrap().0, "S_START");
        assert_ne!(sprites.last().unwrap().0, "S_END");
        assert_eq!(sprites.len(), 483);
        assert_eq!(sprites.get_index(100).unwrap().0, "SARGB5");

        assert!(wad.lumps_between("S_END", "S_START").is_err());

        Ok(())
    }

    #[test]
    fn lumps_after_bounds() -> io::Result<()> {
        let wad = test_wad("doom.wad")?;

        assert!(wad.lumps_after("E1M1", 0).is_ok());
        assert!(wad.lumps_after("E1M1", 9999).is_err());

        Ok(())
    }

    fn test_wad(path: impl AsRef<Path>) -> io::Result<WadFile> {
        WadFile::open(Path::new("test").join(path))
    }
}
