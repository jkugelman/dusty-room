use byteorder::{LittleEndian, ReadBytesExt};
use indexmap::IndexMap;
use std::{
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
    wad_type: WadType,
    lumps: Vec<Lump>,
    lump_indices: HashMap<String, LumpIndex>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum WadType {
    Iwad,
    Pwad,
}

struct Lump {
    name: String,
    contents: Arc<[u8]>,
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

        let (wad_type, lump_count, directory_offset) = read_header(&mut file)?;
        let lump_locations = read_directory(&mut file, directory_offset, lump_count)?;
        let lumps = read_lumps(&lump_locations, &mut file)?;
        let lump_indices = build_indices(&lump_locations);

        Ok(WadFile {
            path: path.to_owned(),
            wad_type,
            lumps,
            lump_indices,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn wad_type(&self) -> WadType {
        self.wad_type
    }

    /// Retrieves a named lump. The name must be unique.
    pub fn lump(&self, name: &str) -> Option<Arc<[u8]>> {
        self.lump_index(name).map(|i| self.lump_contents(i))
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
    /// let map = wad.lumps_after("E1M5", 10);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn lumps_after(&self, start: &str, size: usize) -> Option<LumpBlock> {
        let start_index = self.lump_index(start)? + 1;
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
    /// let sprites = wad.lumps_between("S_START", "S_END");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn lumps_between(&self, start: &str, end: &str) -> Option<LumpBlock> {
        let start_index = self.lump_index(start)? + 1;
        let end_index = self.lump_index(end)?;

        if start_index >= end_index {
            return None;
        }

        let size = end_index - start_index;
        self.lump_block(start_index, size)
    }

    fn lump_index(&self, name: &str) -> Option<usize> {
        self.lump_indices.get(name).and_then(|&index| match index {
            LumpIndex::Unique(index) => Some(index),
            LumpIndex::NotUnique => None,
        })
    }

    fn lump_contents(&self, index: usize) -> Arc<[u8]> {
        self.lumps[index].contents.clone()
    }

    fn lump_block(&self, start_index: usize, size: usize) -> Option<LumpBlock> {
        if start_index + size >= self.lumps.len() {
            return None;
        }

        let mut lumps = IndexMap::with_capacity(size);

        for index in start_index..start_index + size {
            let name = &self.lumps[index].name;
            lumps.insert(name.clone(), self.lump_contents(index));
        }

        Some(lumps)
    }
}

#[derive(Debug)]
struct LumpLocation {
    offset: u32,
    size: u32,
    name: String,
}

/// Reads wad type, lump count, and directory offset.
fn read_header(mut file: impl Read + Seek) -> io::Result<(WadType, u32, u32)> {
    file.seek(SeekFrom::Start(0))?;

    let wad_type = read_wad_type(&mut file)?;
    let lump_count = file.read_u32::<LittleEndian>()?;
    let directory_offset = file.read_u32::<LittleEndian>()?;

    Ok((wad_type, lump_count, directory_offset))
}

fn read_wad_type(mut file: impl Read) -> io::Result<WadType> {
    let mut buffer = [0u8; 4];
    file.read_exact(&mut buffer)?;

    match &buffer {
        b"IWAD" => Ok(WadType::Iwad),
        b"PWAD" => Ok(WadType::Pwad),

        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{:?} neither IWAD nor PWAD", buffer),
        )),
    }
}

fn read_directory(
    mut file: impl Read + Seek,
    directory_offset: u32,
    lump_count: u32,
) -> io::Result<Vec<LumpLocation>> {
    file.seek(SeekFrom::Start(directory_offset.into()))?;

    let mut lump_locations = Vec::with_capacity(lump_count.try_into().unwrap());

    for _ in 0..lump_count {
        let offset = file.read_u32::<LittleEndian>()?;
        let size = file.read_u32::<LittleEndian>()?;
        let mut name = [0u8; 8];
        file.read_exact(&mut name)?;
        let name = std::str::from_utf8(&name)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
            .trim_end_matches('\0')
            .to_owned();

        lump_locations.push(LumpLocation { offset, size, name });
    }

    Ok(lump_locations)
}

fn read_lumps(
    lump_locations: &[LumpLocation],
    mut file: impl Read + Seek,
) -> Result<Vec<Lump>, io::Error> {
    let mut lumps = Vec::new();

    for lump_location in lump_locations {
        let mut contents = vec![0u8; lump_location.size.try_into().unwrap()];
        file.seek(SeekFrom::Start(lump_location.offset.into()))?;
        file.read_exact(&mut contents)?;
        lumps.push(Lump {
            name: lump_location.name.clone(),
            contents: contents.into_boxed_slice().into(),
        });
    }

    Ok(lumps)
}

/// Create map of names to indices. Store `NotUnique` if a name is duplicated.
fn build_indices(lump_locations: &[LumpLocation]) -> HashMap<String, LumpIndex> {
    let mut lump_indices = HashMap::new();

    for (index, lump_location) in lump_locations.iter().enumerate() {
        lump_indices
            .entry(lump_location.name.clone())
            .and_modify(|e| *e = LumpIndex::NotUnique)
            .or_insert(LumpIndex::Unique(index));
    }

    lump_indices
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn header() {
        let wad = test_wad("doom.wad");
        assert_eq!(wad.wad_type, WadType::Iwad);
        assert_eq!(wad.lumps.len(), 1264);

        let wad = test_wad("killer.wad");
        assert_eq!(wad.wad_type, WadType::Pwad);
        assert_eq!(wad.lumps.len(), 55);
    }

    #[test]
    fn read_lumps() {
        let wad = test_wad("doom.wad");

        assert_eq!(wad.lump("DEMO1").unwrap().len(), 20118);
        assert_eq!(wad.lump("E1M1").unwrap().len(), 0);
    }

    #[test]
    fn detect_duplicates() {
        let wad = test_wad("doom.wad");

        assert!(wad.lump_index("E1M1").is_some());
        assert!(wad.lump_index("THINGS").is_none());
        assert!(wad.lump_index("VERTEXES").is_none());
        assert!(wad.lump_index("SECTORS").is_none());
    }

    #[test]
    fn lumps_after() {
        let wad = test_wad("doom.wad");

        let map = wad.lumps_after("E1M8", 10).unwrap();
        assert_eq!(map.len(), 10);
        assert_eq!(
            map.keys().collect::<Vec<_>>(),
            [
                "THINGS", "LINEDEFS", "SIDEDEFS", "VERTEXES", "SEGS", "SSECTORS", "NODES",
                "SECTORS", "REJECT", "BLOCKMAP"
            ],
        );
    }

    #[test]
    fn lumps_between() {
        let wad = test_wad("doom.wad");

        let sprites = wad.lumps_between("S_START", "S_END").unwrap();
        assert_ne!(sprites.first().unwrap().0, "S_START");
        assert_ne!(sprites.last().unwrap().0, "S_END");
        assert_eq!(sprites.len(), 483);
        assert_eq!(sprites.get_index(100).unwrap().0, "SARGB5");

        assert!(wad.lumps_between("S_END", "S_START").is_none());
    }

    #[test]
    fn lumps_after_bounds() {
        let wad = test_wad("doom.wad");

        assert!(wad.lumps_after("E1M1", 0).is_some());
        assert!(wad.lumps_after("E1M1", 9999).is_none());
    }

    fn test_wad(path: impl AsRef<Path>) -> WadFile {
        WadFile::open(Path::new("test").join(path)).unwrap()
    }
}
