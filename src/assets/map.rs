pub use sector::*;
pub use sidedef::*;
pub use vertex::*;

mod sector;
mod sidedef;
mod vertex;

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::assets::{FlatBank, TextureBank};
use crate::wad::{self, Lump, Wad};

#[derive(Debug)]
pub struct MapBank {
    maps: BTreeMap<String, Arc<Map>>,
}

impl MapBank {
    pub fn load(_wad: &Wad, _texture_bank: &TextureBank) -> wad::Result<Self> {
        let maps = BTreeMap::new();

        Ok(Self { maps })
    }

    pub fn get(&self, name: &str) -> Option<&Arc<Map>> {
        self.maps.get(name)
    }
}

pub struct Map {
    name: String,
    things: (),
    vertexes: Vec<Vertex>,
    sidedefs: Sidedefs,
    linedefs: (),
    sectors: Sectors,
}

impl Map {
    /// Loads a map, typically named `ExMy` for DOOM or `MAPnn` for DOOM II.
    ///
    /// # Errors
    ///
    /// Returns `Ok(None)` if the map is missing.
    pub fn load(
        wad: &Wad,
        name: &str,
        flat_bank: &FlatBank,
        texture_bank: &TextureBank,
    ) -> wad::Result<Option<Self>> {
        let lumps = match wad.try_lumps_following(name, 11)? {
            Some(lumps) => lumps,
            None => return Ok(None),
        };

        let name = lumps.name().to_owned();
        let things = Self::read_things(lumps[1].expect_name("THINGS")?);
        let vertexes = Vertex::load(lumps[4].expect_name("VERTEXES")?)?;
        let sectors = Sectors::load(&lumps, flat_bank)?;
        let sidedefs = Sidedefs::load(&lumps, texture_bank)?;
        let linedefs = Self::read_linedefs(lumps[2].expect_name("LINEDEFS")?);

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
    fn read_sectors(_lump: &Lump) {}
    fn read_linedefs(_lump: &Lump) {}
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
        let texture_bank = TextureBank::load(&DOOM_WAD).unwrap();
        let maps = MapBank::load(&DOOM_WAD, &texture_bank).unwrap();
        assert_matches!(maps.get("E1M1"), Some(_));
        assert_matches!(maps.get("E9M9"), None);

        let texture_bank = TextureBank::load(&DOOM2_WAD).unwrap();
        let maps = MapBank::load(&DOOM2_WAD, &texture_bank).unwrap();
        assert_matches!(maps.get("MAP31"), Some(_));
        assert_matches!(maps.get("MAP99"), None);
    }
}
