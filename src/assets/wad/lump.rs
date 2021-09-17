use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::{fmt, slice, vec};

use super::wad::{self, WadFile};

/// A named lump of data from a [`WadFile`].
///
/// This type is not publicly visible. [`Wad`]'s public interface hides `Lump`s behind [`LumpRef`]s.
#[derive(Clone, Debug)]
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

    /// The lump name, for example `VERTEXES` or `THINGS`.
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

    /// Returns `true` if the lump contains no data.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Checks that the lump has the expected name.
    pub fn expect_name(self, name: &str) -> wad::Result<Self> {
        if self.name == name {
            Ok(self)
        } else {
            Err(self.error(&format!("{} missing", name)))
        }
    }

    /// Checks that the lump is the expected number of bytes.
    pub fn expect_size(self, size: usize) -> wad::Result<Self> {
        if self.size() == size {
            Ok(self)
        } else {
            Err(self.error(&format!("expected {} bytes, got {}", size, self.size())))
        }
    }

    /// Creates a [`wad::Error::Malformed`] blaming this lump.
    pub fn error(&self, desc: &str) -> wad::Error {
        self.file.error(&format!("{}: {}", self.name(), desc))
    }
}

impl<'a> fmt::Debug for LumpRef<'a> {
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

impl<'a> fmt::Display for LumpRef<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

/// A block of lumps from a [`Wad`] file.
///
/// Usually the first lump gives the name of the block.
#[derive(Clone, Debug)]
pub struct LumpRefs<'a> {
    lumps: Vec<LumpRef<'a>>,
}

impl<'a> LumpRefs<'a> {
    fn new(lumps: Vec<LumpRef<'a>>) -> Self {
        assert!(lumps.len() > 0);
        Self { lumps }
    }

    /// The path of the file containing the lumps.
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

    /// Gets the lump at `index` and checks that it has the expected name.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get_with_name(&self, index: usize, name: &str) -> wad::Result<LumpRef<'a>> {
        self[index].expect_name(name)
    }

    /// Creates a [`wad::Error::Malformed`] blaming this block.
    pub fn error(&self, desc: &str) -> wad::Error {
        self.lumps
            .first()
            .expect("empty lump block")
            .file
            .error(desc)
    }
}

impl<'a> Deref for LumpRefs<'a> {
    type Target = Vec<LumpRef<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.lumps
    }
}

impl<'a> DerefMut for LumpRefs<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lumps
    }
}

impl<'a> IntoIterator for LumpRefs<'a> {
    type Item = LumpRef<'a>;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.lumps.into_iter()
    }
}

impl<'a, 'b> IntoIterator for &'a LumpRefs<'b> {
    type Item = &'a LumpRef<'b>;
    type IntoIter = slice::Iter<'a, LumpRef<'b>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'b> IntoIterator for &'a mut LumpRefs<'b> {
    type Item = &'a mut LumpRef<'b>;
    type IntoIter = slice::IterMut<'a, LumpRef<'b>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
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
        LumpRefs::new(
            self.iter()
                .map(|lump| LumpRef {
                    file,
                    name: &lump.name,
                    data: &lump.data,
                })
                .collect(),
        )
    }
}
