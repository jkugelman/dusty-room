use bytes::Buf;

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
        let mut cursor = lump.cursor();

        let mut vertexes = Vec::with_capacity(lump.size() / 4);

        while !cursor.has_remaining() {
            let x = cursor.need(2)?.get_i16_le();
            let y = cursor.need(2)?.get_i16_le();
            vertexes.push(Self::new(x, y));
        }

        Ok(vertexes)
    }
}
