use std::fmt;

use crate::map;
use crate::wad::{Lump, Wad};

pub struct Map {
    name: String,
}

impl Map {
    /// Load the named map, typically `"ExMy"` for DOOM or `"MAPnn"` for DOOM II.
    ///
    /// It is an error if the map is missing.
    pub fn load(wad: &Wad, name: &str) -> map::Result<Self> {
        Self::load_lumps(wad.lumps_following(name, 11)?)
    }

    /// Load the named map, typically `"ExMy"` for DOOM or `"MAPnn"` for DOOM II.
    ///
    /// Returns `Ok(None)` if the map is missing.
    pub fn try_load(wad: &Wad, name: &str) -> map::Result<Option<Self>> {
        wad.try_lumps_following(name, 11)?
            .map(Self::load_lumps)
            .transpose()
    }

    fn load_lumps(lumps: &[Lump]) -> map::Result<Self> {
        Ok(Map {
            name: lumps[0].name.clone(),
        })
    }
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
    use crate::{map, test::*};

    #[test]
    fn load() {
        assert_matches!(Map::load(&*DOOM_WAD, "E1M1"), Ok(_));
        assert_matches!(Map::load(&*DOOM_WAD, "E9M9"), Err(map::Error::Wad { .. }));

        assert_matches!(Map::try_load(&*DOOM_WAD, "E1M1"), Ok(Some(_)));
        assert_matches!(Map::try_load(&*DOOM_WAD, "E9M9"), Ok(None));
    }
}
