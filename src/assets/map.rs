pub use linedef::*;
pub use sector::*;
pub use sidedef::*;
pub use vertex::*;

mod linedef;
mod sector;
mod sidedef;
mod vertex;

use std::fmt;

use crate::wad::{self, Lump, Wad};

pub struct Map {
    name: String,
    things: (),
    vertexes: Vertexes,
    sidedefs: Sidedefs,
    linedefs: Linedefs,
    sectors: Sectors,
}

impl Map {
    /// Loads a map, typically named `ExMy` for DOOM or `MAPnn` for DOOM II.
    ///
    /// # Errors
    ///
    /// Returns `Ok(None)` if the map is missing.
    pub fn load(wad: &Wad, name: &str) -> wad::Result<Option<Self>> {
        let lumps = match wad.try_lumps_following(name, 11)? {
            Some(lumps) => lumps,
            None => return Ok(None),
        };

        let name = name.to_owned();
        let things = Self::read_things(lumps[1].expect_name("THINGS")?);
        let vertexes = Vertexes::load(&lumps)?;
        let sectors = Sectors::load(&lumps)?;
        let sidedefs = Sidedefs::load(&lumps)?;
        let linedefs = Linedefs::load(&lumps)?;

        Ok(Some(Map {
            name,
            things,
            vertexes,
            sidedefs,
            linedefs,
            sectors,
        }))
    }

    fn read_things(_lump: &Lump) {}
}

impl fmt::Debug for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            name,
            things,
            vertexes,
            sidedefs,
            linedefs,
            sectors,
        } = self;

        fmt.debug_struct("Map")
            .field("name", &name)
            .field("things", &things)
            .field("vertexes", &vertexes)
            .field("sidedefs", &sidedefs)
            .field("linedefs", &linedefs)
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
        assert_matches!(Map::load(&DOOM_WAD, "E1M1").unwrap(), Some(_));
        assert_matches!(Map::load(&DOOM_WAD, "E9M9").unwrap(), None);

        assert_matches!(Map::load(&DOOM2_WAD, "MAP31").unwrap(), Some(_));
        assert_matches!(Map::load(&DOOM2_WAD, "MAP99").unwrap(), None);
    }
}
