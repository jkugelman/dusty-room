use std::convert::TryInto;
use std::ops::Index;

use bytes::Buf;

use crate::assets::{Assets, Flat};
use crate::wad::{self, Lumps};

/// A list of [sectors] for a particular [map], indexed by number.
///
/// [sectors]: Sector
/// [map]: crate::map::Map
#[derive(Debug)]
pub struct Sectors(Vec<Sector>);

impl Sectors {
    /// Loads a map's sectors from its `SECTORS` lump.
    pub fn load(lumps: &Lumps, assets: &Assets) -> wad::Result<Self> {
        let lump = lumps[8].expect_name("SECTORS")?;

        let mut sectors = Vec::with_capacity(lump.size() / 26);
        let mut cursor = lump.cursor();

        while cursor.has_remaining() {
            // Helper function to verify a flat name.
            let flat_name = |name: String, which: &str| -> wad::Result<String> {
                assets.flat_bank.get(&name).ok_or_else(|| {
                    lump.error(format!(
                        "sector #{} has invalid {} flat {:?}",
                        sectors.len(),
                        which,
                        name
                    ))
                })?;

                Ok(name)
            };

            cursor.need(26)?;
            let floor_height = cursor.get_i16_le();
            let ceiling_height = cursor.get_i16_le();
            let floor_flat = flat_name(cursor.get_name(), "floor")?;
            let ceiling_flat = flat_name(cursor.get_name(), "ceiling")?;
            let light_level: u8 = cursor.get_u16_le().try_into().unwrap_or(u8::MAX);
            let special_type = cursor.get_u16_le();
            let tag = cursor.get_u16_le();

            sectors.push(Sector {
                floor_height,
                ceiling_height,
                floor_flat,
                ceiling_flat,
                light_level,
                special_type,
                tag,
            })
        }

        cursor.done()?;

        Ok(Self(sectors))
    }

    /// Looks up a sector number.
    pub fn get(&self, number: u16) -> Option<&Sector> {
        self.0.get(usize::from(number))
    }
}

impl Index<u16> for Sectors {
    type Output = Sector;

    /// Looks up a sector number.
    fn index(&self, number: u16) -> &Self::Output {
        self.get(number).expect("sector missing")
    }
}

impl std::ops::Deref for Sectors {
    type Target = Vec<Sector>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A horizontal (east-west and north-south) area of the [map] where a floor height and ceiling
/// height are defined. Its shape its defined by its [sidedefs]. Any change in floor or ceiling
/// height or [texture] requires a new sector (and therefore separating [linedefs] and sidedefs).
///
/// [map]: crate::map::Map
/// [sidedefs]: crate::map::Sidedef
/// [texture]: crate::assets::Flat
/// [linedefs]: crate::map::Linedef
#[derive(Debug)]
pub struct Sector {
    /// Floor height.
    pub floor_height: i16,

    /// Ceiling height.
    pub ceiling_height: i16,

    /// Name of the [flat] used for the floor texture.
    ///
    /// [flat]: crate::assets::Flat
    pub floor_flat: String,

    /// Name of the [flat] used for the ceiling texture.
    ///
    /// [flat]: crate::assets::Flat
    pub ceiling_flat: String,

    /// Light level from 0 (total dark) to 255 (maximum brightness). There are actually only 32
    /// brightnesses possible: 0-7 are the same, ..., 248-255 are the same.
    pub light_level: u8,

    pub special_type: u16,

    /// A tag number. When [linedefs] with the same tag number are activated something will usually
    /// happen to this sector: its floor will rise, the lights will go out, etc.
    ///
    /// [linedefs]: crate::map::Linedef
    pub tag: u16,
}

impl Sector {
    /// Looks up the sector's floor flat.
    pub fn floor_flat<'assets>(&self, assets: &'assets Assets) -> &'assets Flat {
        &assets.flat_bank[&self.floor_flat]
    }

    /// Looks up the sector's ceiling flat.
    pub fn ceiling_flat<'assets>(&self, assets: &'assets Assets) -> &'assets Flat {
        &assets.flat_bank[&self.ceiling_flat]
    }
}
