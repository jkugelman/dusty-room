use bytes::Buf;

use crate::wad::{self, Lumps};

#[derive(Debug)]
pub struct Sidedefs(Vec<Sidedef>);

impl Sidedefs {
    pub fn load(lumps: &Lumps) -> wad::Result<Self> {
        let lump = lumps[3].expect_name("SIDEDEFS")?;
        let mut cursor = lump.cursor();

        let mut sidedefs = Vec::with_capacity(lump.size() / 30);

        while cursor.has_remaining() {
            cursor.need(30)?;
            let x_offset = cursor.get_i16_le();
            let y_offset = cursor.get_i16_le();
            let upper_texture = optional(cursor.get_name());
            let lower_texture = optional(cursor.get_name());
            let middle_texture = optional(cursor.get_name());
            let sector = cursor.get_u16_le();

            sidedefs.push(Sidedef {
                x_offset,
                y_offset,
                upper_texture,
                lower_texture,
                middle_texture,
                sector,
            })
        }

        cursor.done()?;

        Ok(Self(sidedefs))
    }
}

fn optional(name: String) -> Option<String> {
    match name.as_str() {
        "-" => None,
        _ => Some(name),
    }
}

/// A `Sidedef` is a definition of what wall [`Texture`]s to draw along a [`Linedef`], and a group
/// of `Sidedef`s outline the space of a [`Sector`].
#[derive(Clone, Debug)]
pub struct Sidedef {
    pub x_offset: i16,
    pub y_offset: i16,
    pub upper_texture: Option<String>,
    pub lower_texture: Option<String>,
    pub middle_texture: Option<String>,
    pub sector: u16,
}
