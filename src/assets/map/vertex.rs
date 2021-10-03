use bytes::Buf;

use crate::wad::{self, Lumps};

/// A list of [`Vertex`]es indexed by number. Each map has unique vertexes.
///
/// Wannabe pedants should note that according to [Merriam-Webster] the plural of "vertex" can be
/// either "vertices" or "vertexes". In this codebase we use id Software's spelling.
///
/// [Merriam-Webster]: https://www.merriam-webster.com/dictionary/vertex
#[derive(Debug)]
pub struct Vertexes(Vec<Vertex>);

impl Vertexes {
    pub fn load(lumps: &Lumps) -> wad::Result<Self> {
        let lump = lumps[4].expect_name("VERTEXES")?;
        let mut cursor = lump.cursor();

        let mut vertexes = Vec::with_capacity(lump.size() / 4);

        while cursor.has_remaining() {
            cursor.need(4)?;
            let x = cursor.get_i16_le();
            let y = cursor.get_i16_le();
            vertexes.push(Vertex { x, y });
        }

        cursor.done()?;

        Ok(Self(vertexes))
    }
}

/// Vertexes are the start and end points of [linedefs] and [segs].
///
/// [linedefs]: crate::assets::Linedef
/// [segs]: crate::assets::Seg
#[derive(Clone, Debug)]
pub struct Vertex {
    pub x: i16,
    pub y: i16,
}
