use std::fmt;
use std::ops::Deref;
use std::path::Path;

use crate::wad::{self, WadFile};

/// A named lump of data from a [`WadFile`].
///
/// This type is not publicly visible. They are stored within [`WadFile`]s and referenced by
/// [`LumpRef`]s and that's it.
#[derive(Debug)]
pub(super) struct Lump {
    pub name: String,
    pub data: Vec<u8>,
}

/// A reference to a lump of data in a [`Wad`] file.
///
/// [`Wad`]: crate::wad::Wad
#[derive(Clone, Copy)]
pub struct LumpRef<'wad> {
    file: &'wad WadFile,
    name: &'wad str,
    data: &'wad [u8],
}

impl LumpRef<'_> {
    /// The path of the file containing the lump.
    pub fn file(&self) -> &Path {
        self.file.path()
    }

    /// The lump name, for example `"VERTEXES"` or `"THINGS"`.
    pub fn name(&self) -> &str {
        self.name
    }

    /// The lump data, a binary blob.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// The size of the lump. Equivalent to `lump.data().len()`.
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Checks that the lump has the expected name.
    pub fn expect(self, name: &str) -> wad::Result<Self> {
        if self.name == name {
            Ok(self)
        } else {
            Err(wad::Error::malformed(
                self.file(),
                format!("{} missing", name),
            ))
        }
    }
}

impl<'wad> fmt::Debug for LumpRef<'wad> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{} from {} ({} bytes)",
            self.name,
            self.file,
            self.size()
        )
    }
}

impl<'wad> fmt::Display for LumpRef<'wad> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

/// A block of lumps from a [`Wad`] file.
///
/// Usually the first lump gives the name of the block.
#[derive(Clone, Debug)]
pub struct LumpRefs<'wad> {
    lumps: Vec<LumpRef<'wad>>,
}

impl<'wad> LumpRefs<'wad> {
    /// The path of the file containing the lump.
    ///
    /// # Panics
    ///
    /// Panics if the block is empty.
    pub fn file(&self) -> &Path {
        self.lumps.first().expect("empty lump block").file()
    }

    /// The name of the block, the name of the first lump.
    ///
    /// # Panics
    ///
    /// Panics if the block is empty.
    pub fn name(&self) -> &str {
        self.lumps.first().expect("empty lump block").name()
    }

    /// Checks that the lump at `index` has the expected name.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get_named(&self, index: usize, name: &str) -> wad::Result<LumpRef<'wad>> {
        let lump = self[index];

        if lump.name == name {
            Ok(lump)
        } else {
            Err(wad::Error::malformed(
                self.file(),
                format!("{} missing {}", self.name(), name),
            ))
        }
    }
}

impl<'wad> Deref for LumpRefs<'wad> {
    type Target = [LumpRef<'wad>];

    fn deref(&self) -> &[LumpRef<'wad>] {
        &self.lumps
    }
}

/// This trait helps [`lookup`] and [`try_lookup`] annotate a lump or block of lumps with the path
/// of the file that contains them.
///
/// [`lookup`]: Wad::lookup
/// [`try_lookup`]: Wad::try_lookup
pub(super) trait FromFile<'file> {
    type Out;
    fn from_file(self, file: &'file WadFile) -> Self::Out;
}

/// Convert a `&Lump` into a `LumpRef` with the path of the file containing the lump.
impl<'file> FromFile<'file> for &'file Lump {
    type Out = LumpRef<'file>;

    fn from_file(self, file: &'file WadFile) -> LumpRef {
        LumpRef {
            file,
            name: &self.name,
            data: &self.data,
        }
    }
}

/// Convert a `&[Lump]` into a `Vec<LumpRef>` with the path of the file containing the lumps.
impl<'file> FromFile<'file> for &'file [Lump] {
    type Out = LumpRefs<'file>;

    fn from_file(self, file: &'file WadFile) -> LumpRefs<'file> {
        LumpRefs {
            lumps: self
                .iter()
                .map(|lump| LumpRef {
                    file,
                    name: &lump.name,
                    data: &lump.data,
                })
                .collect(),
        }
    }
}
