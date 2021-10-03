use bytes::Buf;

use crate::wad::{self, Lumps};

/// A list of [`Linedef`]s indexed by number. Each map has unique linedefs.
#[derive(Debug)]
pub struct Linedefs(Vec<Linedef>);

impl Linedefs {
    /// Loads a map's linedefs from its `LINEDEFS` lump.
    pub fn load(lumps: &Lumps) -> wad::Result<Self> {
        let lump = lumps[2].expect_name("LINEDEFS")?;
        let mut cursor = lump.cursor();

        let mut linedefs = Vec::with_capacity(lump.size() / 14);

        while cursor.has_remaining() {
            cursor.need(14)?;
            let start_vertex = cursor.get_u16_le();
            let end_vertex = cursor.get_u16_le();
            let flags = cursor.get_u16_le();
            let types = cursor.get_u16_le();
            let tag = cursor.get_u16_le();
            let right_sidedef = cursor.get_u16_le();
            let left_sidedef = optional(cursor.get_u16_le());

            linedefs.push(Linedef {
                start_vertex,
                end_vertex,
                flags,
                types,
                tag,
                right_sidedef,
                left_sidedef,
            })
        }

        cursor.done()?;

        Ok(Self(linedefs))
    }
}

fn optional(sidedef: u16) -> Option<u16> {
    match sidedef {
        u16::MAX => None,
        _ => Some(sidedef),
    }
}

/// A `Linedef` represents a one- or two-sided line between two [vertexes]. Each linedef has
/// optional left and right [sidedefs] that link to the adjoining [sector] or sectors.
///
/// [vertexes]: crate::assets::Vertex
/// [sidedefs]: crate::assets::Sidedef
/// [sector]: crate::assets::Sector
#[derive(Clone, Debug)]
pub struct Linedef {
    /// Number of the starting vertex.
    pub start_vertex: u16,

    /// Number of the ending vertex.
    pub end_vertex: u16,

    pub flags: u16,

    pub types: u16,

    /// A tag number which ties this line's trigger effect to all [sectors] with a matching tag number.
    ///
    /// [sectors]: crate::assets::Sector
    pub tag: u16,

    /// Number of the right sidedef, where "right" is based on the direction of the linedef from the
    /// start vertex to the end vertex. All lines have a right side.
    pub right_sidedef: u16,

    /// If this is a two-sided line, number of the left sidedef, where "left" is based on the
    /// direction of the linedef from the start vertex to the end vertex.
    pub left_sidedef: Option<u16>,
}
