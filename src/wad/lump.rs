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
