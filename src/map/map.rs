use std::fmt;

use crate::map;
use crate::wad::{LumpRef, Wad};

pub struct Map {
    name: String,
    things: (),
    vertices: (),
    sides: (),
    lines: (),
    sectors: (),
}

impl Map {
    /// Load the named map, typically `"ExMy"` for DOOM or `"MAPnn"` for DOOM II.
    ///
    /// Returns `Ok(None)` if the map is missing.
    pub fn load(wad: &Wad, name: &str) -> map::Result<Option<Self>> {
        let lumps = wad.try_lumps_following(name, 11)?;
        if lumps.is_none() {
            return Ok(None);
        }
        let lumps = lumps.unwrap();

        let name = name.to_string();
        let things = Self::read_things(lumps.get_named(1, "THINGS")?);
        let vertices = Self::read_vertices(lumps.get_named(4, "VERTEXES")?);
        let sectors = Self::read_sectors(lumps.get_named(8, "SECTORS")?);
        let sides = Self::read_sides(lumps.get_named(3, "SIDEDEFS")?);
        let lines = Self::read_lines(lumps.get_named(2, "LINEDEFS")?);

        Ok(Some(Map {
            name,
            things,
            vertices,
            sides,
            lines,
            sectors,
        }))
    }

    fn read_things(_lump: LumpRef) {}
    fn read_vertices(_lump: LumpRef) {}
    fn read_sectors(_lump: LumpRef) {}
    fn read_sides(_lump: LumpRef) {}
    fn read_lines(_lump: LumpRef) {}
}

impl fmt::Debug for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{:?}", self.name)
    }
}

impl fmt::Display for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;

    #[test]
    fn load() {
        assert_matches!(Map::load(&*DOOM_WAD, "E1M1"), Ok(Some(_)));
        assert_matches!(Map::load(&*DOOM_WAD, "E9M9"), Ok(None));
    }
}
