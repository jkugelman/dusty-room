use std::convert::TryInto;

use crate::wad::{self, Lump};

#[derive(Clone, Debug)]
pub struct Vertex {
    x: i16,
    y: i16,
}

impl Vertex {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub fn load(lump: &Lump) -> wad::Result<Vec<Self>> {
        assert_eq!(lump.name(), "VERTEXES");

        let count = lump.expect_size_multiple(4)?.size() / 4;
        let mut vertexes = Vec::with_capacity(count);

        for chunk in lump.data().chunks_exact(4) {
            let x = i16::from_le_bytes(chunk[0..2].try_into().unwrap());
            let y = i16::from_le_bytes(chunk[2..4].try_into().unwrap());
            vertexes.push(Self::new(x, y));
        }

        Ok(vertexes)
    }
}
