use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    collections::HashMap,
    convert::TryInto,
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use crate::{Lump, Wad};

/// A single IWAD or PWAD file.
pub struct WadFile {
    path: PathBuf,
    wad_type: WadType,
    lumps: Vec<Lump>,
    lump_indices: HashMap<String, LumpIndex>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WadType {
    Iwad,
    Pwad,
}

#[derive(Clone, Copy, Debug)]
enum LumpIndex {
    Unique(usize),
    NotUnique,
}

impl WadFile {
    /// Reads a WAD file from disk.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::open_impl(path.as_ref())
    }

    fn open_impl(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut file = BufReader::new(file);

        let (wad_type, lump_count, directory_offset) = read_header(&mut file)?;
        let lump_locations = read_directory(&mut file, directory_offset, lump_count)?;
        let lump_indices = build_indices(&lump_locations);
        let lumps = read_lumps(lump_locations, &mut file)?;

        Ok(WadFile {
            path: path.into(),
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

    fn lump_index(&self, name: &str) -> Option<usize> {
        self.lump_indices.get(name).and_then(|&index| match index {
            LumpIndex::Unique(index) => Some(index),
            LumpIndex::NotUnique => None,
        })
    }
}

impl Wad for WadFile {
    /// Retrieves a named lump. The name must be unique.
    fn lump(&self, name: &str) -> Option<&Lump> {
        self.lump_index(name).map(|i| &self.lumps[i])
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
    fn lumps_after(&self, start: &str, size: usize) -> Option<&[Lump]> {
        let start_index = self.lump_index(start)? + 1;
        self.lumps.get(start_index..start_index + size)
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
    fn lumps_between(&self, start: &str, end: &str) -> Option<&[Lump]> {
        let start_index = self.lump_index(start)? + 1;
        let end_index = self.lump_index(end)?;

        self.lumps.get(start_index..end_index)
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
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?
            .trim_end_matches('\0')
            .to_string();

        lump_locations.push(LumpLocation { offset, size, name });
    }

    Ok(lump_locations)
}

fn read_lumps(
    locations: Vec<LumpLocation>,
    mut file: impl Read + Seek,
) -> Result<Vec<Lump>, io::Error> {
    let mut lumps = Vec::new();

    for location in locations {
        file.seek(SeekFrom::Start(location.offset.into()))?;
        let mut data = vec![0u8; location.size.try_into().unwrap()];
        file.read_exact(&mut data)?;

        lumps.push(Lump {
            name: location.name,
            data,
        });
    }

    Ok(lumps)
}

/// Create map of names to indices. Store `NotUnique` if a name is duplicated.
fn build_indices(locations: &[LumpLocation]) -> HashMap<String, LumpIndex> {
    let mut indices = HashMap::new();

    for (index, location) in locations.iter().enumerate() {
        indices
            .entry(location.name.clone())
            .and_modify(|e| *e = LumpIndex::NotUnique)
            .or_insert(LumpIndex::Unique(index));
    }

    indices
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

        assert_eq!(wad.lump("DEMO1").unwrap().size(), 20118);
        assert_eq!(wad.lump("E1M1").unwrap().size(), 0);
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
            map.iter().map(|l| &l.name).collect::<Vec<_>>(),
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
        assert_ne!(sprites.first().unwrap().name, "S_START");
        assert_ne!(sprites.last().unwrap().name, "S_END");
        assert_eq!(sprites.len(), 483);
        assert_eq!(sprites[100].name, "SARGB5");

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
