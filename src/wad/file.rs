use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::{fmt, io};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::wad::{self, Lump, ResultExt};

/// A single IWAD or PWAD file.
///
/// This is a low level type. Most code should use [`Wad`].
///
/// [`Wad`]: crate::wad::Wad
pub struct WadFile {
    path: PathBuf,
    wad_type: WadType,
    lumps: Vec<Lump>,
    lump_indices: HashMap<String, Vec<usize>>,
}

#[derive(Debug)]
struct Header {
    pub wad_type: WadType,
    pub lump_count: u32,
    pub directory_offset: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WadType {
    Iwad,
    Pwad,
}

#[derive(Debug)]
struct Directory {
    pub lump_locations: Vec<LumpLocation>,
}

#[derive(Debug)]
struct LumpLocation {
    pub offset: u32,
    pub size: u32,
    pub name: String,
}

impl fmt::Display for LumpLocation {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{} (offset {}, size {})",
            self.name, self.offset, self.size
        )
    }
}

impl WadFile {
    /// Reads a WAD file from disk.
    pub fn open(path: impl AsRef<Path>) -> wad::Result<Self> {
        Self::open_impl(path.as_ref())
    }

    fn open_impl(path: &Path) -> wad::Result<Self> {
        let result: io::Result<Self> = (|| {
            let file = File::open(path)?;
            let mut file = BufReader::new(file);

            let Header {
                wad_type,
                lump_count,
                directory_offset,
            } = Self::read_header(&mut file)?;

            let Directory { lump_locations } =
                Self::read_directory(&mut file, directory_offset, lump_count)?;

            let mut wad_file = WadFile {
                path: path.into(),
                wad_type,
                lumps: Vec::new(),
                lump_indices: HashMap::new(),
            };
            wad_file.build_indices(&lump_locations);
            wad_file.read_lumps(&mut file, lump_locations)?;

            Ok(wad_file)
        })();

        result.err_path(path)
    }

    fn read_header(mut file: impl Read + Seek) -> io::Result<Header> {
        file.seek(SeekFrom::Start(0))?;

        let wad_type = Self::read_wad_type(&mut file)?;
        let lump_count = file.read_u32::<LittleEndian>()?;
        let directory_offset = file.read_u32::<LittleEndian>()?;

        Ok(Header {
            wad_type,
            lump_count,
            directory_offset,
        })
    }

    fn read_wad_type(file: impl Read) -> io::Result<WadType> {
        let mut buffer = Vec::new();
        file.take(4).read_to_end(&mut buffer)?;

        match &buffer[..] {
            b"IWAD" => Ok(WadType::Iwad),
            b"PWAD" => Ok(WadType::Pwad),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "not a WAD file")),
        }
    }

    fn read_directory(
        mut file: impl Read + Seek,
        directory_offset: u32,
        lump_count: u32,
    ) -> io::Result<Directory> {
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

        Ok(Directory { lump_locations })
    }

    fn build_indices(&mut self, locations: &[LumpLocation]) {
        for (index, location) in locations.iter().enumerate() {
            self.lump_indices
                .entry(location.name.clone())
                .and_modify(|indices: &mut Vec<usize>| indices.push(index))
                .or_insert(vec![index]);
        }
    }

    fn read_lumps(
        &mut self,
        mut file: impl Read + Seek,
        locations: Vec<LumpLocation>,
    ) -> io::Result<()> {
        for location in locations {
            let LumpLocation { offset, size, name } = location;

            file.seek(SeekFrom::Start(offset.into()))?;
            let mut data = vec![0u8; size.try_into().unwrap()];
            file.read_exact(&mut data)?;

            self.lumps.push(Lump { name, data });
        }

        Ok(())
    }

    /// The file's path on disk.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns whether this is an IWAD or PWAD.
    pub fn wad_type(&self) -> WadType {
        self.wad_type
    }

    /// Retrieves a unique lump by name.
    ///
    /// It is an error if the lump is missing.
    pub fn lump(&self, name: &str) -> wad::Result<&Lump> {
        self.try_lump(name)?
            .ok_or_else(|| wad::Error::malformed(&self.path, format!("{} missing", name)))
    }

    /// Retrieves a unique lump by name.
    ///
    /// It is not an error if the lump is missing.
    pub fn try_lump(&self, name: &str) -> wad::Result<Option<&Lump>> {
        let index = self.try_lump_index(name)?;
        if index.is_none() {
            return Ok(None);
        }
        let index = index.unwrap();

        Ok(Some(&self.lumps[index]))
    }

    /// Retrieves a block of `size` lumps following a unique named marker. The marker lump is
    /// included in the result.
    ///
    /// It is an error if the block is missing.
    pub fn lumps_following(&self, start: &str, size: usize) -> wad::Result<&[Lump]> {
        self.try_lumps_following(start, size)?
            .ok_or_else(|| wad::Error::malformed(&self.path, format!("{} missing", start)))
    }

    /// Retrieves a block of `size` lumps following a unique named marker. The marker lump is
    /// included in the result.
    ///
    /// It is not an error if the block is missing.
    pub fn try_lumps_following(&self, start: &str, size: usize) -> wad::Result<Option<&[Lump]>> {
        let start_index = self.try_lump_index(start)?;
        if start_index.is_none() {
            return Ok(None);
        }
        let start_index = start_index.unwrap();

        if start_index + size >= self.lumps.len() {
            return Err(wad::Error::malformed(
                &self.path,
                format!("{} missing lumps", start),
            ));
        }

        Ok(Some(&self.lumps[start_index..start_index + size]))
    }

    /// Retrieves a block of lumps between unique start and end markers. The marker lumps are
    /// included in the result.
    ///
    /// It is an error if the block is missing.
    pub fn lumps_between(&self, start: &str, end: &str) -> wad::Result<&[Lump]> {
        self.try_lumps_between(start, end)?.ok_or_else(|| {
            wad::Error::malformed(&self.path, format!("{} and {} missing", start, end))
        })
    }

    /// Retrieves a block of lumps between unique start and end markers. The marker lumps are
    /// included in the result.
    ///
    /// It is not an error if the block is missing.
    pub fn try_lumps_between(&self, start: &str, end: &str) -> wad::Result<Option<&[Lump]>> {
        let start_index = self.try_lump_index(start)?;
        let end_index = self.try_lump_index(end)?;

        match (start_index, end_index) {
            (Some(_), Some(_)) => {}

            (None, None) => {
                return Ok(None);
            }

            (Some(_), None) => {
                return Err(wad::Error::malformed(
                    &self.path,
                    format!("{} without {}", start, end),
                ));
            }

            (None, Some(_)) => {
                return Err(wad::Error::malformed(
                    &self.path,
                    format!("{} without {}", end, start),
                ));
            }
        }

        let start_index = start_index.unwrap();
        let end_index = end_index.unwrap();

        if start_index > end_index {
            return Err(wad::Error::malformed(
                &self.path,
                format!("{} after {}", start, end),
            ));
        }

        Ok(Some(&self.lumps[start_index..end_index + 1]))
    }

    /// Looks up a lump's index. It's an error if the lump isn't unique.
    fn try_lump_index(&self, name: &str) -> wad::Result<Option<usize>> {
        let indices: Option<&[usize]> = self.lump_indices.get(name).map(Vec::as_slice);

        match indices {
            Some(&[index]) => Ok(Some(index)),
            Some(indices) => Err(wad::Error::malformed(
                &self.path,
                format!("{} found {} times", name, indices.len()),
            )),
            None => Ok(None),
        }
    }
}

impl fmt::Debug for WadFile {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{:?}", self.path)
    }
}

impl fmt::Display for WadFile {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.path.display())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;

    #[test]
    fn header() {
        assert_eq!(DOOM_WAD_FILE.wad_type, WadType::Iwad);
        assert_eq!(DOOM_WAD_FILE.lumps.len(), 1264);

        assert_eq!(KILLER_WAD_FILE.wad_type, WadType::Pwad);
        assert_eq!(KILLER_WAD_FILE.lumps.len(), 55);

        assert_matches!(
            WadFile::open("test/killer.txt"),
            Err(wad::Error::Io { source: err, ..}) if err.kind() == io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn read_lumps() {
        assert_eq!(DOOM_WAD_FILE.lump("DEMO1").unwrap().size(), 20118);
        assert_eq!(DOOM_WAD_FILE.lump("E1M1").unwrap().size(), 0);
    }

    #[test]
    fn detect_duplicates() {
        assert_matches!(DOOM_WAD_FILE.lump("E1M1"), Ok(_));
        assert_matches!(DOOM_WAD_FILE.lump("THINGS"), Err(_));
        assert_matches!(DOOM_WAD_FILE.lump("VERTEXES"), Err(_));
        assert_matches!(DOOM_WAD_FILE.lump("SECTORS"), Err(_));
    }

    #[test]
    fn lumps_after() {
        let map = DOOM_WAD_FILE.lumps_following("E1M8", 11).unwrap();
        assert_eq!(map.len(), 11);
        assert_eq!(
            map.iter().map(|lump| &lump.name).collect::<Vec<_>>(),
            [
                "E1M8", "THINGS", "LINEDEFS", "SIDEDEFS", "VERTEXES", "SEGS", "SSECTORS", "NODES",
                "SECTORS", "REJECT", "BLOCKMAP"
            ],
        );
    }

    #[test]
    fn lumps_between() {
        let sprites = DOOM_WAD_FILE.lumps_between("S_START", "S_END").unwrap();
        assert_eq!(sprites.first().unwrap().name, "S_START");
        assert_eq!(sprites.last().unwrap().name, "S_END");
        assert_eq!(sprites.len(), 485);
        assert_eq!(sprites[100].name, "SARGB4B6");

        assert_matches!(DOOM_WAD_FILE.lumps_between("S_END", "S_START"), Err(_));
    }

    #[test]
    fn lumps_after_bounds() {
        assert_matches!(DOOM_WAD_FILE.try_lumps_following("E1M1", 0), Ok(Some(_)));
        assert_matches!(DOOM_WAD_FILE.try_lumps_following("E1M1", 9999), Err(_));
    }
}
