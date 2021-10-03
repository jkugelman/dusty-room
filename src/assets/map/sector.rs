use std::convert::TryInto;

use bytes::Buf;

use crate::wad::{self, Lumps};

/// A list of [sectors] for a particular [map], indexed by number.
///
/// [sectors]: Sector
/// [map]: crate::assets::Map
#[derive(Debug)]
pub struct Sectors(Vec<Sector>);

impl Sectors {
    /// Loads a map's sectors from its `SECTORS` lump.
    pub fn load(lumps: &Lumps) -> wad::Result<Self> {
        let lump = lumps[8].expect_name("SECTORS")?;
        let mut cursor = lump.cursor();

        let mut sectors = Vec::with_capacity(lump.size() / 26);

        while cursor.has_remaining() {
            cursor.need(26)?;
            let floor_height = cursor.get_i16_le();
            let ceiling_height = cursor.get_i16_le();
            let floor_flat = cursor.get_name();
            let ceiling_flat = cursor.get_name();
            let light_level = cursor.get_u16_le().try_into().unwrap_or(u8::MAX);
            let special_type = cursor.get_u16_le();
            let tag_number = cursor.get_u16_le();

            sectors.push(Sector {
                floor_height,
                ceiling_height,
                floor_flat,
                ceiling_flat,
                light_level,
                special_type,
                tag_number,
            })
        }

        cursor.done()?;

        Ok(Self(sectors))
    }
}

/// A horizontal (east-west and north-south) area of the [map] where a floor height and ceiling
/// height are defined. Its shape its defined by its [sidedefs]. Any change in floor or ceiling
/// height or [texture] requires a new sector (and therefore separating [linedefs] and sidedefs).
///
/// [map]: crate::assets::Map
/// [sidedefs]: crate::assets::Sidedef
/// [texture]: crate::assets::Texture
/// [linedefs]: crate::assets::Linedef
#[derive(Debug)]
pub struct Sector {
    /// Floor height.
    pub floor_height: i16,

    /// Ceiling height.
    pub ceiling_height: i16,

    /// Name of the flat used for the floor texture.
    pub floor_flat: String,

    /// Name of the flat used for the ceiling texture.
    pub ceiling_flat: String,

    /// Light level of this sector, from 0 (total dark) to 255 (maximum brightness). There are
    /// actually only 32 brightnesses possible: 0-7 are the same, ..., 248-255 are the same.
    pub light_level: u8,

    pub special_type: u16,

    /// A tag number. When [linedefs] with the same tag number are activated something will usually
    /// happen to this sector: its floor will rise, the lights will go out, etc.
    ///
    /// [linedefs]: crate::assets::Linedef
    pub tag_number: u16,
}
