use std::fmt;

use crate::wad::{self, Lump, Wad};

pub struct Map {
    name: String,
    _things: (),
    _vertices: (),
    _sides: (),
    _lines: (),
    _sectors: (),
}

impl Map {
    /// Loads a map, typically named `ExMy` for DOOM or `MAPnn` for DOOM II.
    ///
    /// Returns `Ok(None)` if the map is missing.
    pub fn load(wad: &Wad, name: &str) -> wad::Result<Option<Self>> {
        let lumps = wad.try_lumps_following(name, 11)?;
        if lumps.is_none() {
            return Ok(None);
        }
        let lumps = lumps.unwrap();

        let name = name.to_owned();
        let things = Self::read_things(lumps[1].expect_name("THINGS")?);
        let vertices = Self::read_vertices(lumps[4].expect_name("VERTEXES")?);
        let sectors = Self::read_sectors(lumps[8].expect_name("SECTORS")?);
        let sides = Self::read_sides(lumps[3].expect_name("SIDEDEFS")?);
        let lines = Self::read_lines(lumps[2].expect_name("LINEDEFS")?);

        Ok(Some(Map {
            name,
            _things: things,
            _vertices: vertices,
            _sides: sides,
            _lines: lines,
            _sectors: sectors,
        }))
    }

    fn read_things(_lump: &Lump) {}
    fn read_vertices(_lump: &Lump) {}
    fn read_sectors(_lump: &Lump) {}
    fn read_sides(_lump: &Lump) {}
    fn read_lines(_lump: &Lump) {}
}

impl fmt::Debug for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            name,
            _things,
            _vertices,
            _sides,
            _lines,
            _sectors,
        } = self;

        write!(fmt, "{:?}", name)
    }
}

impl fmt::Display for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wad::test::*;

    #[test]
    fn load() {
        assert_matches!(Map::load(&*DOOM_WAD, "E1M1"), Ok(Some(_)));
        assert_matches!(Map::load(&*DOOM_WAD, "E9M9"), Ok(None));
    }
}
