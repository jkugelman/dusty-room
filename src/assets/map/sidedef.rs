use std::ops::Deref;

use bytes::Buf;

use crate::assets::Sectors;
use crate::wad::{self, Lumps};

/// A list of [sidedefs] for a particular [map], indexed by number.
///
/// [sidedefs]: Sidedef
/// [map]: crate::assets::Map
#[derive(Debug)]
pub struct Sidedefs(Vec<Sidedef>);

impl Sidedefs {
    /// Loads a map's sidedefs from its `SIDEDEFS` lump.
    pub fn load(lumps: &Lumps, sectors: &Sectors) -> wad::Result<Self> {
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

            sectors.get(usize::from(sector)).ok_or_else(|| {
                lump.error(format!(
                    "sidedef #{} has invalid sector #{}",
                    sidedefs.len(),
                    sector
                ))
            })?;

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

impl Deref for Sidedefs {
    type Target = Vec<Sidedef>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A description of what wall [textures] to draw along a [linedef]. A group of sidedefs outlines
/// the space of a [sector].
///
/// [textures]: crate::assets::Texture
/// [linedef]: crate::assets::Linedef
/// [sector]: crate::assets::Sector
#[derive(Clone, Debug)]
pub struct Sidedef {
    /// X offset to start at when drawing the wall textures. A positive offset moves them left
    /// so the left sides get cut off. A negative offset moves them right.
    pub x_offset: i16,

    /// Y offset to start at when drawing the wall textures. A positive offset moves them up
    /// so the top edges get cut off. A negative offset moves them down.
    pub y_offset: i16,

    /// Optional upper [texture] name, if the adjacent [sector]'s ceiling is lower.
    ///
    /// [texture]: crate::assets::Texture
    /// [sector]: crate::assets::Sector
    pub upper_texture: Option<String>,

    /// Optional lower [texture] name, if the adjacent [sector]'s floor is higher.
    ///
    /// [texture]: crate::assets::Texture
    /// [sector]: crate::assets::Sector
    pub lower_texture: Option<String>,

    /// Optional middle [texture] name. One-sided linedefs should always have a middle texture.
    /// Two-sided linedefs are usually transparent, though they sometimes have partially see-through
    /// textures such as for fences or windows.
    ///
    /// [texture]: crate::assets::Texture
    pub middle_texture: Option<String>,

    /// [Sector] number this sidedef faces or helps to surround.
    ///
    /// [sector]: crate::assets::Sector
    pub sector: u16,
}
