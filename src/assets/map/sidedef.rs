use std::sync::Arc;

use bytes::Buf;

use crate::assets::{Texture, TextureBank};
use crate::wad::{self, Lumps};

#[derive(Debug)]
pub struct Sidedefs(Vec<Sidedef>);

impl Sidedefs {
    pub fn load(lumps: &Lumps, texture_bank: &TextureBank) -> wad::Result<Self> {
        let lump = lumps[3].expect_name("SIDEDEFS")?;
        let mut cursor = lump.cursor();

        let mut sidedefs = Vec::with_capacity(lump.size() / 30);

        while cursor.has_remaining() {
            // Helper function to look up textures. `-` means no texture.
            let get_texture = |name: &str| -> wad::Result<Option<Arc<Texture>>> {
                if name == "-" {
                    Ok(None)
                } else {
                    let texture = texture_bank
                        .get(name)
                        .ok_or_else(|| {
                            lumps.error(format!(
                                "SIDEDEF #{} needs missing texture {}",
                                sidedefs.len(),
                                name
                            ))
                        })?
                        .clone();
                    Ok(Some(texture))
                }
            };

            cursor.need(30)?;
            let x_offset = cursor.get_i16_le();
            let y_offset = cursor.get_i16_le();
            let upper_texture = get_texture(&cursor.get_name())?;
            let lower_texture = get_texture(&cursor.get_name())?;
            let middle_texture = get_texture(&cursor.get_name())?;
            let sector_index = cursor.get_u16_le();

            sidedefs.push(Sidedef {
                x_offset,
                y_offset,
                upper_texture,
                lower_texture,
                middle_texture,
                sector_index,
            })
        }

        cursor.done()?;

        Ok(Self(sidedefs))
    }
}

/// A `Sidedef` is a definition of what wall [`Texture`]s to draw along a [`Linedef`], and a group
/// of `Sidedef`s outline the space of a [`Sector`].
#[derive(Clone, Debug)]
pub struct Sidedef {
    x_offset: i16,
    y_offset: i16,
    upper_texture: Option<Arc<Texture>>,
    lower_texture: Option<Arc<Texture>>,
    middle_texture: Option<Arc<Texture>>,
    sector_index: u16,
}
