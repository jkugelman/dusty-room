use std::borrow::Cow;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::{fmt, slice, vec};

use bytes::Bytes;

use crate::wad::{self, Cursor, WadFile};

/// A block of one or more [`Lump`]s from a [`Wad`] or [`WadFile`].
///
/// [`Wad`]: crate::wad::Wad
#[derive(Clone, Debug)]
pub struct Lumps(Vec<Lump>);

impl Lumps {
    /// Creates a block of lumps.
    ///
    /// # Panics
    ///
    /// Panics if `lumps` is empty.
    pub(super) fn new(mut lumps: Vec<Lump>, is_named: bool) -> Self {
        assert!(!lumps.is_empty());

        if is_named {
            let name = lumps[0].name.clone();

            for lump in lumps.iter_mut() {
                *lump = lump.from_block(name.clone());
            }
        }

        Self(lumps)
    }

    /// The file containing the lumps.
    pub fn file(&self) -> &Arc<WadFile> {
        // It doesn't matter which lump we look at. They all come from the same file.
        self.first().file()
    }

    /// The first lump in the block.
    pub fn first(&self) -> &Lump {
        self.0.first().unwrap()
    }

    /// The last lump in the block.
    pub fn last(&self) -> &Lump {
        self.0.last().unwrap()
    }

    /// Creates a [`wad::Error::Malformed`] blaming this block.
    pub fn error(&self, desc: impl Into<Cow<'static, str>>) -> wad::Error {
        self.first().file.error(desc)
    }
}

impl Deref for Lumps {
    type Target = Vec<Lump>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Lumps {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Lumps {
    type Item = Lump;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Lumps {
    type Item = &'a Lump;
    type IntoIter = slice::Iter<'a, Lump>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Lumps {
    type Item = &'a mut Lump;
    type IntoIter = slice::IterMut<'a, Lump>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A lump of data from a [`Wad`] or [`WadFile`].
///
/// [`Wad`]: crate::wad::Wad
#[derive(Clone)]
pub struct Lump {
    file: Arc<WadFile>,
    block: Option<String>,
    name: String,
    data: Bytes,
}

impl Lump {
    /// Creates a lump pointing at a slice of data from a `WadFile`.
    pub(super) fn new(file: Arc<WadFile>, name: String, data: Bytes) -> Self {
        Self {
            file,
            block: None,
            name,
            data,
        }
    }

    pub(super) fn from_block(&self, block: String) -> Self {
        Self {
            block: Some(block),
            ..self.clone()
        }
    }

    /// The file containing the lump.
    pub fn file(&self) -> &Arc<WadFile> {
        &self.file
    }

    /// The lump name, for example `VERTEXES` or `THINGS`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The lump data, a binary blob.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns a cursor that can be used to parse the lump data.
    pub fn cursor(&self) -> Cursor<'_> {
        Cursor::new(self, self.data.clone())
    }

    /// The size of the lump.
    ///
    /// This is equivalent to `self.data().len()`.
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if this is a marker lump with no data.
    ///
    /// This is equivalent to `self.data.len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns `true` if the lump contains data.
    ///
    /// This is equivalent to `!self.is_empty()`.
    pub fn has_data(&self) -> bool {
        !self.is_empty()
    }

    /// Checks that the lump has the expected name.
    pub fn expect_name(&self, name: &str) -> wad::Result<&Self> {
        if self.name == name {
            Ok(self)
        } else {
            Err(self.error(format!("{} missing", name)))
        }
    }

    /// Creates a [`wad::Error::Malformed`] blaming this lump.
    pub fn error(&self, desc: impl Into<Cow<'static, str>>) -> wad::Error {
        self.file.error(format!("{}: {}", self.name(), desc.into()))
    }
}

impl fmt::Debug for Lump {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            file,
            block,
            name,
            data,
        } = self;

        if let Some(block) = block {
            write!(fmt, "{} ", block)?;
        }
        write!(fmt, "{} ({} bytes) from {}", name, data.len(), file)
    }
}

impl fmt::Display for Lump {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref block) = self.block {
            write!(fmt, "{} ", block)?;
        }
        write!(fmt, "{}", self.name)
    }
}
