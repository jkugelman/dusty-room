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

/// Contains all the level geometry, monsters, items, and other things that make up a map.
#[derive(Debug)]
pub struct Map {
    /// Map name such as `E1M1` or `MAP01`.
    pub name: String,

    /// A list of things indexed by number.
    pub things: (),

    /// A list of vertexes indexed by number.
    pub vertexes: Vertexes,

    /// A list of sidedefs indexed by number.
    pub sidedefs: Sidedefs,

    /// A list of linedefs indexed by number.
    pub linedefs: Linedefs,

    /// A list of sectors indexed by number.
    pub sectors: Sectors,
}

impl Map {
    /// Loads a map. Maps are typically named `ExMy` for DOOM or `MAPnn` for DOOM II.
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
        let sidedefs = Sidedefs::load(&lumps, &sectors)?;
        let linedefs = Linedefs::load(&lumps, &vertexes, &sidedefs)?;

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
