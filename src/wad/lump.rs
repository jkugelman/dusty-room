use std::fmt::{self, Display};

#[allow(unused_imports)]
use crate::Wad;

/// A named lump of data from a [`Wad`] file.
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ({} bytes)", self.name, self.size())
    }
}

impl Display for Lump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
