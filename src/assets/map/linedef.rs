use bytes::Buf;

use crate::wad::{self, Lumps};

#[derive(Debug)]
pub struct Linedefs(Vec<Linedef>);

impl Linedefs {
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
            let left_sidedef = optional(cursor.get_u16_le());
            let right_sidedef = optional(cursor.get_u16_le());

            linedefs.push(Linedef {
                start_vertex,
                end_vertex,
                flags,
                types,
                tag,
                left_sidedef,
                right_sidedef,
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
    pub start_vertex: u16,
    pub end_vertex: u16,
    pub flags: u16,
    pub types: u16,
    pub tag: u16,
    pub left_sidedef: Option<u16>,
    pub right_sidedef: Option<u16>,
}
