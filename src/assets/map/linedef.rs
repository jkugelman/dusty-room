use bytes::Buf;

use crate::assets::{Sidedefs, Vertexes};
use crate::wad::{self, Lumps};

/// A list of [linedefs] for a particular [map], indexed by number.
///
/// [linedefs]: Linedef
/// [map]: crate::assets::Map
#[derive(Debug)]
pub struct Linedefs(Vec<Linedef>);

impl Linedefs {
    /// Loads a map's linedefs from its `LINEDEFS` lump.
    pub fn load(lumps: &Lumps, vertexes: &Vertexes, sidedefs: &Sidedefs) -> wad::Result<Self> {
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
            let right_sidedef = optional(cursor.get_u16_le()).ok_or_else(|| {
                lump.error(format!("linedef #{} missing right sidedef", linedefs.len()))
            })?;
            let left_sidedef = optional(cursor.get_u16_le());

            vertexes.get(usize::from(start_vertex)).ok_or_else(|| {
                lump.error(format!(
                    "linedef #{} has invalid start vertex #{}",
                    linedefs.len(),
                    start_vertex
                ))
            })?;

            vertexes.get(usize::from(end_vertex)).ok_or_else(|| {
                lump.error(format!(
                    "linedef #{} has invalid end vertex #{}",
                    linedefs.len(),
                    end_vertex
                ))
            })?;

            sidedefs.get(usize::from(right_sidedef)).ok_or_else(|| {
                lump.error(format!(
                    "linedef #{} has invalid right sidedef #{}",
                    linedefs.len(),
                    right_sidedef
                ))
            })?;

            if let Some(left_sidedef) = left_sidedef {
                sidedefs.get(usize::from(left_sidedef)).ok_or_else(|| {
                    lump.error(format!(
                        "linedef #{} has invalid left sidedef #{}",
                        linedefs.len(),
                        left_sidedef
                    ))
                })?;
            }

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
    /// Starting [vertex] number.
    ///
    /// [vertex]: crate::assets::Vertex
    pub start_vertex: u16,

    /// Ending [vertex] number.
    ///
    /// [vertex]: crate::assets::Vertex
    pub end_vertex: u16,

    pub flags: u16,

    pub types: u16,

    /// A tag number which ties this line's trigger effect to all [sectors] with a matching tag
    /// number.
    ///
    /// [sectors]: crate::assets::Sector
    pub tag: u16,

    /// Right [sidedef] number, where "right" is based on the direction of the linedef from the
    /// start vertex to the end vertex. All lines have a right side.
    ///
    /// [sidedef]: crate::assets::Sidedef
    pub right_sidedef: u16,

    /// Left [sidedef] number if this is a two-sided line, where "left" is based on the direction of
    /// the linedef from the start vertex to the end vertex.
    ///
    /// [sidedef]: crate::assets::Sidedef
    pub left_sidedef: Option<u16>,
}
