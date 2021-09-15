use std::fmt;

/// A named lump of data from a [`Wad`] file.
///
/// [`Wad`]: crate::Wad
pub struct Lump {
    /// The lump name, for example `"VERTEXES"` or `"THINGS"`.
    pub name: String,
    /// The lump contents, a binary blob.
    pub data: Vec<u8>,
}

impl Lump {
    /// The size of the lump. Equivalent to `lump.data.len()`.
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl fmt::Debug for Lump {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{:?} ({} bytes)", self.name, self.size())
    }
}

impl fmt::Display for Lump {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}
