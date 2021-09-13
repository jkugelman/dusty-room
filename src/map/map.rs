use std::fmt;

use crate::map;
use crate::wad::Wad;

pub struct Map {
    name: String,
}

impl Map {
    pub fn load(wad: &Wad, name: &str) -> map::Result<Self> {
        let _lumps = wad.lumps_following(name, 11)?;

        Ok(Self { name: name.into() })
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
        assert_matches!(
            Map::load(&*DOOM_WAD, "E9M9"),
            Err(map::Error::WadError { .. })
        );
    }
}
