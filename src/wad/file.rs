use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::TryInto;

use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::{fmt, io};

use lazy_static::lazy_static;
use regex::Regex;

use crate::wad::{self, Lump, Lumps, ResultExt};

/// A single IWAD or PWAD.
///
/// This is a lower level type. Code outside the [`wad`] module should mainly use the [`Wad`]
/// struct, which has a similar interface with the added capability of being able to add patch WADs
/// on top of the base game WAD.
///
/// [`Wad`]: crate::wad::Wad
pub struct WadFile {
    path: PathBuf,
    raw: Vec<u8>,
    kind: WadKind,
    lump_locations: Vec<LumpLocation>,
    lump_indices: HashMap<String, Vec<usize>>,
}

#[derive(Debug)]
struct Header {
    pub kind: WadKind,
    pub lump_count: usize,
    pub directory_offset: usize,
}

/// WAD files can be either IWADs or PWADs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WadKind {
    /// An IWAD or "internal wad" such as `doom.wad` that contains all of the data necessary to
    /// play.
    Iwad,
    /// A PWAD or "patch wad" containing extra levels, textures, or other assets that are overlaid
    /// on top of other wads.
    Pwad,
}

#[derive(Debug)]
struct Directory {
    pub lump_locations: Vec<LumpLocation>,
    pub lump_indices: HashMap<String, Vec<usize>>,
}

#[derive(Debug)]
struct LumpLocation {
    pub offset: usize,
    pub size: usize,
    pub name: String,
}

lazy_static! {
    static ref LUMP_NAME_REGEX: Regex = Regex::new(r"^[A-Z0-9\[\]\-_\\]+$").unwrap();
}

impl WadFile {
    /// Loads a WAD file from disk.
    pub fn load(path: impl AsRef<Path>) -> wad::Result<Self> {
        let path = path.as_ref();
        let file = File::open(path).err_path(path)?;
        Self::load_reader(path, file)
    }

    /// Loads a WAD file from a generic reader.
    ///
    /// The reader's current position doesn't matter. Reading WAD files requires seeking to
    /// arbitrary offsets throughout the file.
    ///
    /// The `path` only used for display purposes, such as in error messages. It doesn't need to
    /// point to an actual file on disk.
    pub fn load_reader(path: impl AsRef<Path>, file: impl Read + Seek) -> wad::Result<Self> {
        let path = path.as_ref();
        let raw = Self::read_into_vec(file).err_path(path)?;
        Self::load_raw(path, raw)
    }

    fn read_into_vec(mut file: impl Read + Seek) -> io::Result<Vec<u8>> {
        // If the file is really large it may not fit into memory. Individual allocations can never
        // exceed `isize::MAX` bytes, which is just 2GB on a 32-bit system.
        //
        // This won't catch all panics. Ideally we could check if `Vec::with_capacity` fails, but in
        // stable Rust there's no way to do that. Nightly offers `Vec::try_reserve`, so hope is on
        // the horizon.
        let size = file.seek(SeekFrom::End(0))?;
        if isize::try_from(size).is_err() {
            return Err(io::Error::new(io::ErrorKind::OutOfMemory, "file too large"));
        }
        let size: usize = size.try_into().unwrap();
        let mut raw = Vec::with_capacity(size);

        file.rewind()?;
        file.read_to_end(&mut raw)?;

        Ok(raw)
    }

    /// Loads a WAD file from a raw byte buffer.
    ///
    /// The `path` only used for display purposes, such as in error messages. It doesn't need to
    /// point to an actual file on disk.
    pub fn load_raw(path: impl AsRef<Path>, raw: Vec<u8>) -> wad::Result<Self> {
        Self::load_raw_impl(path.as_ref(), raw)
            .map_err(|desc: String| wad::Error::malformed(path, desc))
    }

    // Non-generic helper to minimize the amount of code subject to monomorphization.
    fn load_raw_impl(path: &Path, raw: Vec<u8>) -> Result<Self, String> {
        let Header {
            kind,
            lump_count,
            directory_offset,
        } = Self::read_header(&raw)?;

        let Directory {
            lump_locations,
            lump_indices,
        } = Self::read_directory(&raw, lump_count, directory_offset)?;

        Ok(Self {
            path: path.to_owned(),
            raw,
            kind,
            lump_locations,
            lump_indices,
        })
    }

    fn read_header(raw: &[u8]) -> Result<Header, String> {
        let raw = raw.get(0..12).ok_or_else(|| "not a WAD file".to_owned())?;

        let kind = match &raw[0..4] {
            b"IWAD" => WadKind::Iwad,
            b"PWAD" => WadKind::Pwad,
            _ => return Err("not a WAD file".to_owned()),
        };
        let lump_count = u32::from_le_bytes(raw[4..8].try_into().unwrap());
        let directory_offset = u32::from_le_bytes(raw[8..12].try_into().unwrap());

        Ok(Header {
            kind,
            lump_count: lump_count.try_into().unwrap(),
            directory_offset: directory_offset.try_into().unwrap(),
        })
    }

    fn read_directory(
        raw: &[u8],
        lump_count: usize,
        directory_offset: usize,
    ) -> Result<Directory, String> {
        let mut cursor = raw
            .get(directory_offset..)
            .ok_or_else(|| format!("lump directory at illegal offset {}", directory_offset))?;

        // The WAD is untrusted so clamp how much memory is pre-allocated. For comparison,
        // `doom.wad` has 1,264 lumps and `doom2.wad` has 2,919.
        let mut lump_locations = Vec::with_capacity(lump_count.clamp(0, 4096));

        for _ in 0..lump_count {
            let entry = &cursor
                .get(0..16)
                .ok_or_else(|| format!("lump directory has illegal count {}", lump_count))?;

            let offset = u32::from_le_bytes(entry[0..4].try_into().unwrap());
            let size = u32::from_le_bytes(entry[4..8].try_into().unwrap());
            let name: [u8; 8] = entry[8..16].try_into().unwrap();

            // Advance the read cursor.
            cursor = &cursor[16..];

            // Strip trailing NULs and convert into a `String`. Stay away from `str::from_utf8` so
            // we don't have to deal with UTF-8 decoding errors.
            let name = name
                .iter()
                .take_while(|&&b| b != 0u8)
                .map(|&b| b as char)
                .collect::<String>();

            // Verify that the lump name is all uppercase, digits, and a handful of acceptable
            // symbols.
            if !LUMP_NAME_REGEX.is_match(&name) {
                return Err(format!("illegal lump name {:?}", name));
            }

            // Check lump bounds now so we don't have to later.
            let offset: usize = offset.try_into().unwrap();
            let size: usize = size.try_into().unwrap();

            if offset >= raw.len() {
                return Err(format!("{} at illegal offset {}", name, offset));
            }
            if offset + size >= raw.len() {
                return Err(format!("{} has illegal size {}", name, size));
            }

            lump_locations.push(LumpLocation { offset, size, name });
        }

        // Build a map of lump names -> indices for fast lookup.
        let mut lump_indices = HashMap::new();

        for (index, location) in lump_locations.iter().enumerate() {
            lump_indices
                .entry(location.name.clone())
                .and_modify(|indices: &mut Vec<usize>| indices.push(index))
                .or_insert_with(|| vec![index]);
        }

        Ok(Directory {
            lump_locations,
            lump_indices,
        })
    }

    /// The file's path on disk.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns whether this is an IWAD or PWAD.
    pub fn kind(&self) -> WadKind {
        self.kind
    }

    /// Checks that the file is the correct kind.
    pub fn expect_kind(self, expected: WadKind) -> wad::Result<Self> {
        if self.kind() == expected {
            Ok(self)
        } else {
            Err(wad::Error::WrongType {
                path: self.path().to_owned(),
                expected,
            })
        }
    }

    /// Retrieves a unique lump by name.
    ///
    /// # Errors
    ///
    /// It is an error if the lump is missing.
    pub fn lump(&self, name: &str) -> wad::Result<Lump> {
        self.try_lump(name)?
            .ok_or_else(|| self.error(format!("{} missing", name)))
    }

    /// Retrieves a unique lump by name.
    ///
    /// Returns `Ok(None)` if the lump is missing.
    pub fn try_lump(&self, name: &str) -> wad::Result<Option<Lump>> {
        let index = self.try_lump_index(name)?;
        if index.is_none() {
            return Ok(None);
        }
        let index = index.unwrap();

        Ok(Some(self.read_lump(index)?))
    }

    /// Retrieves a block of `size > 0` lumps following a unique named marker. The marker lump is
    /// included in the result.
    ///
    /// # Errors
    ///
    /// It is an error if the block is missing.
    ///
    /// # Panics
    ///
    /// Panics if `size == 0`.
    pub fn lumps_following(&self, start: &str, size: usize) -> wad::Result<Lumps> {
        self.try_lumps_following(start, size)?
            .ok_or_else(|| self.error(format!("{} missing", start)))
    }

    /// Retrieves a block of `size > 0` lumps following a unique named marker. The marker lump is
    /// included in the result.
    ///
    /// Returns `Ok(None)` if the block is missing.
    ///
    /// # Panics
    ///
    /// Panics if `size == 0`.
    pub fn try_lumps_following(&self, start: &str, size: usize) -> wad::Result<Option<Lumps>> {
        assert!(size > 0);

        let start_index = self.try_lump_index(start)?;
        if start_index.is_none() {
            return Ok(None);
        }
        let start_index = start_index.unwrap();

        if start_index + size >= self.lump_indices.len() {
            return Err(self.error(format!("{} missing lumps", start)));
        }

        Ok(Some(self.read_lumps(start_index..start_index + size)?))
    }

    /// Retrieves a block of lumps between unique start and end markers. The marker lumps are
    /// included in the result.
    ///
    /// # Errors
    ///
    /// It is an error if the block is missing.
    pub fn lumps_between(&self, start: &str, end: &str) -> wad::Result<Lumps> {
        self.try_lumps_between(start, end)?
            .ok_or_else(|| self.error(format!("{} and {} missing", start, end)))
    }

    /// Retrieves a block of lumps between unique start and end markers. The marker lumps are
    /// included in the result.
    ///
    /// Returns `Ok(None)` if the block is missing.
    pub fn try_lumps_between(&self, start: &str, end: &str) -> wad::Result<Option<Lumps>> {
        let start_index = self.try_lump_index(start)?;
        let end_index = self.try_lump_index(end)?;

        match (start_index, end_index) {
            (Some(_), Some(_)) => {}

            (None, None) => {
                return Ok(None);
            }

            (Some(_), None) => {
                return Err(self.error(format!("{} without {}", start, end)));
            }

            (None, Some(_)) => {
                return Err(self.error(format!("{} without {}", end, start)));
            }
        }

        let start_index = start_index.unwrap();
        let end_index = end_index.unwrap();

        if start_index > end_index {
            return Err(self.error(format!("{} after {}", start, end)));
        }

        Ok(Some(self.read_lumps(start_index..end_index + 1)?))
    }

    /// Looks up a lump's index. It's an error if the lump isn't unique.
    fn try_lump_index(&self, name: &str) -> wad::Result<Option<usize>> {
        let indices: Option<&[usize]> = self.lump_indices.get(name).map(Vec::as_slice);

        match indices {
            Some(&[index]) => Ok(Some(index)),
            Some(indices) => Err(self.error(format!("{} found {} times", name, indices.len()))),
            None => Ok(None),
        }
    }

    /// Reads a lump from the raw data, pulling out a slice.
    fn read_lump(&self, index: usize) -> wad::Result<Lump> {
        let location = &self.lump_locations[index];

        let file = self;
        let name = &location.name;
        let data = &self.raw[location.offset..][..location.size];

        Ok(Lump::new(file, name, data))
    }

    /// Reads one or more lumps from the raw data, pulling out slices.
    fn read_lumps(&self, indices: Range<usize>) -> wad::Result<Lumps> {
        assert!(!indices.is_empty());

        let lumps: Vec<Lump> = indices
            .map(|index| self.read_lump(index))
            .collect::<wad::Result<_>>()?;

        Ok(Lumps::new(lumps))
    }

    /// Creates a [`wad::Error::Malformed`] blaming this file.
    pub fn error(&self, desc: impl Into<Cow<'static, str>>) -> wad::Error {
        wad::Error::malformed(&self.path, desc)
    }
}

impl fmt::Debug for WadFile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("WadFile")
            .field("path", &self.path)
            .field("raw", &format!("<{} bytes>", self.raw.len()))
            .field("kind", &self.kind)
            .field("lump_locations", &self.lump_locations)
            .field("lump_indices", &self.lump_indices)
            .finish()
    }
}

impl fmt::Display for WadFile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.path.display())
    }
}

#[cfg(test)]
mod tests {
    //! This file is covered by tests in [`crate::wad::wad`].
}
