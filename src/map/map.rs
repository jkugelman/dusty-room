use std::fmt;

use crate::{MapError, Wad};

pub struct Map {
    name: String,
}

impl Map {
    pub fn load(wad: impl Wad, name: &str) -> Result<Self, MapError> {
        Self::load_impl(&wad, name)
    }

    fn load_impl(wad: &dyn Wad, name: &str) -> Result<Self, MapError> {
        let _lumps = wad
            .lumps_after(name, 10)
            .ok_or_else(|| MapError::LumpMissing(name.to_string()))?;

        Ok(Self {
            name: name.to_string(),
        })
    }
}

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map").field("name", &self.name).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test::*, MapError};

    #[test]
    fn load() {
        assert_matches!(Map::load(&*DOOM_WAD, "E1M1"), Ok(_));
        assert_matches!(Map::load(&*DOOM_WAD, "E9M9"), Err(MapError::LumpMissing(name)) if &name == "E9M9");
    }
}
