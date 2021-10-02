use bytes::Buf;

use crate::wad::{self, Lump};

/// Vertexes are the start and end points of [`Linedef`]s and [`Seg`]s.
///
/// Wannabe pedants should note that according to [Merriam-Webster] the plural of "vertex" can be
/// either "vertices" or "vertexes". In this codebase we use id Software's spelling.
///
/// [`Linedef`]: crate::assets::Linedef
/// [`Seg`]: crate::assets::Seg
/// [Merriam-Webster]: https://www.merriam-webster.com/dictionary/vertex
#[derive(Debug)]
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
