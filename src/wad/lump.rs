use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::{fmt, slice, vec};

use crate::wad::{self, WadFile};

/// A reference to a lump of data in a [`Wad`] file.
///
/// [`Wad`]: crate::wad::Wad
#[derive(Clone, Copy)]
pub struct Lump<'wad> {
    pub(super) file: &'wad WadFile,
    pub(super) name: &'wad str,
    pub(super) data: &'wad [u8],
}

impl<'wad> Lump<'wad> {
    /// The path of the file containing the lump.
    pub fn file(&self) -> &'wad Path {
        self.file.path()
    }

    /// The lump name, for example `VERTEXES` or `THINGS`.
    pub fn name(&self) -> &'wad str {
        self.name
    }

    /// The lump data, a binary blob.
    pub fn data(&self) -> &'wad [u8] {
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

    /// Checks that the lump contains a multiple of `size` bytes.
    pub fn expect_size_multiple(self, size: usize) -> wad::Result<Self> {
        if self.size() % size == 0 {
            Ok(self)
        } else {
            Err(self.error(&format!(
                "expected a multiple of {} bytes, got {}",
                size,
                self.size()
            )))
        }
    }
    /// Creates a [`wad::Error::Malformed`] blaming this lump.
    pub fn error(&self, desc: &str) -> wad::Error {
        self.file.error(&format!("{}: {}", self.name(), desc))
    }
}

impl<'a> fmt::Debug for Lump<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{} from {} ({} bytes)",
            self.name,
            self.file,
            self.size()
        )
    }
}

impl<'a> fmt::Display for Lump<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

/// A block of lumps from a [`Wad`] file.
///
/// Usually the first lump gives the name of the block.
#[derive(Clone, Debug)]
pub struct Lumps<'a>(Vec<Lump<'a>>);

impl<'a> Lumps<'a> {
    pub(super) fn new(lumps: Vec<Lump<'a>>) -> Self {
        assert!(lumps.len() > 0);
        Self(lumps)
    }

    /// The path of the file containing the lumps.
    ///
    /// # Panics
    ///
    /// Panics if the block is empty.
    pub fn file(&self) -> &Path {
        self.0.first().expect("empty lump block").file()
    }

    /// The name of the block, the name of the first lump.
    ///
    /// # Panics
    ///
    /// Panics if the block is empty.
    pub fn name(&self) -> &str {
        self.0.first().expect("empty lump block").name()
    }

    /// Gets the lump at `index` and checks that it has the expected name.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get_with_name(&self, index: usize, name: &str) -> wad::Result<Lump<'a>> {
        self[index].expect_name(name)
    }

    /// Creates a [`wad::Error::Malformed`] blaming this block.
    pub fn error(&self, desc: &str) -> wad::Error {
        self.0.first().expect("empty lump block").file.error(desc)
    }
}

impl<'a> Deref for Lumps<'a> {
    type Target = Vec<Lump<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for Lumps<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for Lumps<'a> {
    type Item = Lump<'a>;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, 'b> IntoIterator for &'a Lumps<'b> {
    type Item = &'a Lump<'b>;
    type IntoIter = slice::Iter<'a, Lump<'b>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'b> IntoIterator for &'a mut Lumps<'b> {
    type Item = &'a mut Lump<'b>;
    type IntoIter = slice::IterMut<'a, Lump<'b>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
