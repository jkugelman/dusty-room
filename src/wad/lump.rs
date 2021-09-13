use std::fmt;

/// A named lump of data from a [`Wad`] file.
///
/// [`Wad`]: crate::Wad
pub struct Lump {
    pub name: String,
    pub data: Vec<u8>,
}

impl Lump {
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
