use std::fmt;

use crate::assets::Assets;
use crate::map::{Linedefs, Sectors, Sidedefs, Vertexes};
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
    pub fn load(wad: &Wad, name: &str, assets: &Assets) -> wad::Result<Option<Self>> {
        let lumps = match wad.try_lumps_following(name, 11)? {
            Some(lumps) => lumps,
            None => return Ok(None),
        };

        let name = name.to_owned();
        let things = Self::read_things(lumps[1].expect_name("THINGS")?);
        let vertexes = Vertexes::load(&lumps)?;
        let sectors = Sectors::load(&lumps, assets)?;
        let sidedefs = Sidedefs::load(&lumps, assets, &sectors)?;
        let linedefs = Linedefs::load(&lumps, &vertexes, &sidedefs)?;

        Ok(Some(Map { name, things, vertexes, sidedefs, linedefs, sectors }))
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
    use crate::assets::{Flat, Texture};
    use crate::wad::test::*;

    #[test]
    fn load() {
        let assets = Assets::load(&DOOM_WAD).unwrap();
        assert_matches!(Map::load(&DOOM_WAD, "E1M1", &assets).unwrap(), Some(_));
        assert_matches!(Map::load(&DOOM_WAD, "E9M9", &assets).unwrap(), None);

        let assets = Assets::load(&DOOM2_WAD).unwrap();
        assert_matches!(Map::load(&DOOM2_WAD, "MAP31", &assets).unwrap(), Some(_));
        assert_matches!(Map::load(&DOOM2_WAD, "MAP99", &assets).unwrap(), None);
    }

    #[test]
    fn geometry() {
        let assets = Assets::load(&DOOM2_WAD).unwrap();
        let map =
            Map::load(&DOOM2_WAD, "MAP31", &assets).expect("failed to load").expect("map missing");

        assert_eq!(map.vertexes.len(), 635);
        assert_eq!(map.linedefs.len(), 686);
        assert_eq!(map.sidedefs.len(), 783);
        assert_eq!(map.sectors.len(), 80);

        assert_matches!(
            map.linedefs[42].right_sidedef(&map).middle_texture(&assets),
            Some(Texture { name, width: 128, height: 128, .. }) if name == "ZZWOLF9"
        );

        assert_matches!(
            map.sectors[69].ceiling_flat(&assets),
            Flat { name, .. } if name == "CEIL5_1"
        );
    }
}
