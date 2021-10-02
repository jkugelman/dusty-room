use std::sync::Arc;

use bytes::Buf;

use crate::assets::{Flat, FlatBank};
use crate::wad::{self, Lumps};

#[derive(Debug)]
pub struct Sectors(Vec<Sector>);

impl Sectors {
    pub fn load(lumps: &Lumps, flat_bank: &FlatBank) -> wad::Result<Self> {
        let lump = lumps[8].expect_name("SECTORS")?;
        let mut cursor = lump.cursor();

        let mut sectors = Vec::with_capacity(lump.size() / 26);

        while cursor.has_remaining() {
            // Helper function to look up flats.
            let get_flat = |name: &str| -> wad::Result<Arc<Flat>> {
                let flat = flat_bank
                    .get(name)
                    .ok_or_else(|| {
                        lumps.error(format!(
                            "SECTOR #{} needs missing flat {}",
                            sectors.len(),
                            name
                        ))
                    })?
                    .clone();
                Ok(flat)
            };

            cursor.need(26)?;
            let floor_height = cursor.get_i16_le();
            let ceiling_height = cursor.get_i16_le();
            let floor_flat = get_flat(&cursor.get_name())?;
            let ceiling_flat = get_flat(&cursor.get_name())?;
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
    floor_height: i16,
    ceiling_height: i16,
    floor_flat: Arc<Flat>,
    ceiling_flat: Arc<Flat>,
    light_level: u16,
    kind: u16,
    tag_number: u16,
}
