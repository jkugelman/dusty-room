use std::fmt;

use crate::map;
use crate::wad::Wad;

pub struct Map {
    name: String,
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
        let _lumps = lumps.unwrap();

        Ok(Some(Map { name: name.into() }))
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
    use crate::test::*;

    #[test]
    fn load() {
        assert_matches!(Map::load(&*DOOM_WAD, "E1M1"), Ok(Some(_)));
        assert_matches!(Map::load(&*DOOM_WAD, "E9M9"), Ok(None));
    }
}
