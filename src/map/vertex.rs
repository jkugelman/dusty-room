use std::ops::{Deref, Index};

use bytes::Buf;

use crate::wad::{self, Lumps};

/// A list of [vertexes] for a particular map, indexed by number.
///
/// Wannabe pedants should note that according to [Merriam-Webster] the plural of "vertex" can be
/// either "vertices" or "vertexes". In this codebase we use id Software's spelling.
///
/// [vertexes]: Vertex
/// [map]: crate::map::Map
/// [Merriam-Webster]: https://www.merriam-webster.com/dictionary/vertex
#[derive(Debug)]
pub struct Vertexes(Vec<Vertex>);

impl Vertexes {
    /// Loads a map's vertexes from its `VERTEXES` lump.
    pub fn load(lumps: &Lumps) -> wad::Result<Self> {
        let lump = lumps[4].expect_name("VERTEXES")?;

        let mut vertexes = Vec::with_capacity(lump.size() / 4);
        let mut cursor = lump.cursor();

        while cursor.has_remaining() {
            cursor.need(4)?;
            let x = cursor.get_i16_le();
            let y = cursor.get_i16_le();
            vertexes.push(Vertex { x, y });
        }

        cursor.done()?;

        Ok(Self(vertexes))
    }

    /// Looks up a vertex number.
    pub fn get(&self, number: u16) -> Option<&Vertex> {
        self.0.get(usize::from(number))
    }
}

impl Index<u16> for Vertexes {
    type Output = Vertex;

    /// Looks up a vertex number.
    fn index(&self, number: u16) -> &Self::Output {
        &self.0[usize::from(number)]
    }
}

impl Deref for Vertexes {
    type Target = Vec<Vertex>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Vertexes are the start and end points of [linedefs] and [segs].
///
/// [linedefs]: crate::map::Linedef
/// [segs]: crate::assets::Seg
#[derive(Clone, Debug)]
pub struct Vertex {
    /// X coordinate.
    pub x: i16,

    /// Y coordinate.
    pub y: i16,
}
