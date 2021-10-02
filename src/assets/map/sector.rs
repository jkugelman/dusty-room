use bytes::Buf;

use crate::wad::{self, Lumps};

#[derive(Debug)]
pub struct Sectors(Vec<Sector>);

impl Sectors {
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
            let light_level = cursor.get_u16_le();
            let kind = cursor.get_u16_le();
            let tag_number = cursor.get_u16_le();

            sectors.push(Sector {
                floor_height,
                ceiling_height,
                floor_flat,
                ceiling_flat,
                light_level,
                kind,
                tag_number,
            })
        }

        cursor.done()?;

        Ok(Self(sectors))
    }
}

#[derive(Debug)]
pub struct Sector {
    pub floor_height: i16,
    pub ceiling_height: i16,
    pub floor_flat: String,
    pub ceiling_flat: String,
    pub light_level: u16,
    pub kind: u16,
    pub tag_number: u16,
}
