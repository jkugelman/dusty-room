pub use vertex::*;

mod vertex;

use std::collections::BTreeMap;
use std::fmt;

use crate::wad::{self, Lump, Wad};

#[derive(Debug)]
pub struct MapBank {
    maps: BTreeMap<String, Map>,
}

impl MapBank {
    pub fn load(wad: &Wad) -> wad::Result<Self> {
        let maps = BTreeMap::new();

        Ok(Self { maps })
    }

    pub fn get(&self, name: &str) -> Option<&Map> {
        todo!()
    }
}

pub struct Map {
    name: String,
    things: (),
    vertexes: Vec<Vertex>,
    sides: (),
    lines: (),
    sectors: (),
}

impl Map {
    /// Loads a map, typically named `ExMy` for DOOM or `MAPnn` for DOOM II.
    ///
    /// # Errors
    ///
    /// Returns `Ok(None)` if the map is missing.
    pub fn load(wad: &Wad, name: &str) -> wad::Result<Option<Self>> {
        let lumps = wad.try_lumps_following(name, 11)?;
        if lumps.is_none() {
            return Ok(None);
        }
        let lumps = lumps.unwrap();

        let name = lumps.name().to_owned();
        let things = Self::read_things(lumps[1].expect_name("THINGS")?);
        let vertexes = Vertex::load(lumps[4].expect_name("VERTEXES")?)?;
        let sectors = Self::read_sectors(lumps[8].expect_name("SECTORS")?);
        let sides = Self::read_sides(lumps[3].expect_name("SIDEDEFS")?);
        let lines = Self::read_lines(lumps[2].expect_name("LINEDEFS")?);

        Ok(Some(Map {
            name,
            things,
            vertexes,
            sides,
            lines,
            sectors,
        }))
    }

    fn read_things(_lump: &Lump) {}
    fn read_sectors(_lump: &Lump) {}
    fn read_sides(_lump: &Lump) {}
    fn read_lines(_lump: &Lump) {}
}

impl fmt::Debug for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            name,
            things,
            vertexes,
            sides,
            lines,
            sectors,
        } = self;

        fmt.debug_struct("Map")
            .field("name", &name)
            .field("things", &things)
            .field("vertexes", &vertexes)
            .field("sides", &sides)
            .field("lines", &lines)
            .field("sectors", &sectors)
            .finish()
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
        let maps = MapBank::load(&DOOM_WAD).unwrap();
        assert_matches!(maps.get("E1M1"), Some(_));
        assert_matches!(maps.get("E9M9"), None);

        let maps = MapBank::load(&DOOM2_WAD).unwrap();
        assert_matches!(maps.get("MAP31"), Some(_));
        assert_matches!(maps.get("MAP99"), None);
    }
}
