use std::ops::Index;

use bytes::Buf;

use crate::map::{Map, Sidedef, Sidedefs, Vertex, Vertexes};
use crate::wad::{self, Lumps};

/// A list of [linedefs] for a particular [map], indexed by number.
///
/// [linedefs]: Linedef
/// [map]: crate::map::Map
#[derive(Debug)]
pub struct Linedefs(Vec<Linedef>);

impl Linedefs {
    /// Loads a map's linedefs from its `LINEDEFS` lump.
    pub fn load(lumps: &Lumps, vertexes: &Vertexes, sidedefs: &Sidedefs) -> wad::Result<Self> {
        let lump = lumps[2].expect_name("LINEDEFS")?;

        let mut linedefs = Vec::with_capacity(lump.size() / 14);
        let mut cursor = lump.cursor();

        while cursor.has_remaining() {
            // Helper function to verify a vertex number.
            let vertex_number = |vertex: u16, which: &str| -> wad::Result<u16> {
                vertexes.get(vertex).ok_or_else(|| {
                    lump.error(format!(
                        "linedef #{} has invalid {} vertex #{}",
                        linedefs.len(),
                        which,
                        vertex
                    ))
                })?;
                Ok(vertex)
            };

            // Helper function to verify a sidedef number. `-1` indicates no sidedef.
            let sidedef_number = |sidedef: u16, which: &str| -> wad::Result<Option<u16>> {
                if sidedef == u16::MAX {
                    Ok(None)
                } else {
                    sidedefs.get(sidedef).ok_or_else(|| {
                        lump.error(format!(
                            "linedef #{} has invalid {} sidedef #{}",
                            linedefs.len(),
                            which,
                            sidedef
                        ))
                    })?;
                    Ok(Some(sidedef))
                }
            };

            cursor.need(14)?;
            let start_vertex = vertex_number(cursor.get_u16_le(), "start")?;
            let end_vertex = vertex_number(cursor.get_u16_le(), "end")?;
            let flags = cursor.get_u16_le();
            let types = cursor.get_u16_le();
            let tag = cursor.get_u16_le();
            let right_sidedef = sidedef_number(cursor.get_u16_le(), "right")?.ok_or_else(|| {
                lump.error(format!("linedef #{} missing right sidedef", linedefs.len()))
            })?;
            let left_sidedef = sidedef_number(cursor.get_u16_le(), "left")?;

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

    /// Looks up a linedef number.
    pub fn get(&self, number: u16) -> Option<&Linedef> {
        self.0.get(usize::from(number))
    }
}

impl Index<u16> for Linedefs {
    type Output = Linedef;

    /// Looks up a linedef number.
    fn index(&self, number: u16) -> &Self::Output {
        &self.0[usize::from(number)]
    }
}

/// A `Linedef` represents a one- or two-sided line between two [vertexes]. Each linedef has
/// optional left and right [sidedefs] that link to the adjoining [sector] or sectors.
///
/// [vertexes]: crate::map::Vertex
/// [sidedefs]: crate::map::Sidedef
/// [sector]: crate::map::Sector
#[derive(Clone, Debug)]
pub struct Linedef {
    /// Starting [vertex] number.
    ///
    /// [vertex]: crate::map::Vertex
    pub start_vertex: u16,

    /// Ending [vertex] number.
    ///
    /// [vertex]: crate::map::Vertex
    pub end_vertex: u16,

    pub flags: u16,

    pub types: u16,

    /// A tag number which ties this line's trigger effect to all [sectors] with a matching tag
    /// number.
    ///
    /// [sectors]: crate::map::Sector
    pub tag: u16,

    /// Right [sidedef] number, where "right" is based on the direction of the linedef from the
    /// start vertex to the end vertex. All lines have a right side.
    ///
    /// [sidedef]: crate::map::Sidedef
    pub right_sidedef: u16,

    /// Left [sidedef] number if this is a two-sided line, where "left" is based on the direction of
    /// the linedef from the start vertex to the end vertex.
    ///
    /// [sidedef]: crate::map::Sidedef
    pub left_sidedef: Option<u16>,
}

impl Linedef {
    /// Looks up the linedef's start vertex.
    pub fn start_vertex<'map>(&self, map: &'map Map) -> &'map Vertex {
        &map.vertexes[self.start_vertex]
    }

    /// Looks up the linedef's end vertex.
    pub fn end_vertex<'map>(&self, map: &'map Map) -> &'map Vertex {
        &map.vertexes[self.end_vertex]
    }

    /// Looks up the linedef's right sidedef.
    pub fn right_sidedef<'map>(&self, map: &'map Map) -> &'map Sidedef {
        &map.sidedefs[self.right_sidedef]
    }

    /// Looks up the linedef's left sidedef.
    pub fn left_sidedef<'map>(&self, map: &'map Map) -> Option<&'map Sidedef> {
        Some(&map.sidedefs[self.left_sidedef?])
    }
}
