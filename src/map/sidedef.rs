use std::ops::{Deref, Index};

use bytes::Buf;

use crate::assets::Assets;
use crate::map::{Map, Sector, Sectors};
use crate::wad::{self, Lumps};

/// A list of [sidedefs] for a particular [map], indexed by number.
///
/// [sidedefs]: Sidedef
/// [map]: crate::map::Map
#[derive(Debug)]
pub struct Sidedefs(Vec<Sidedef>);

impl Sidedefs {
    /// Loads a map's sidedefs from its `SIDEDEFS` lump.
    pub fn load(lumps: &Lumps, assets: &Assets, sectors: &Sectors) -> wad::Result<Self> {
        let lump = lumps[3].expect_name("SIDEDEFS")?;

        let mut sidedefs = Vec::with_capacity(lump.size() / 30);
        let mut cursor = lump.cursor();

        while cursor.has_remaining() {
            // Helper function to verify a texture name.
            let texture_name = |name: String, which: &str| -> wad::Result<Option<String>> {
                if name == "-" {
                    return Ok(None);
                }

                assets.texture_bank.get(&name).ok_or_else(|| {
                    lump.error(format!(
                        "sidedef #{} has invalid {} texture {:?}",
                        sidedefs.len(),
                        which,
                        name
                    ))
                })?;

                Ok(Some(name))
            };

            // Helper function to verify a sector number.
            let sector_number = |sector: u16| -> wad::Result<u16> {
                sectors.get(sector).ok_or_else(|| {
                    lump.error(format!(
                        "sidedef #{} has invalid sector #{}",
                        sidedefs.len(),
                        sector
                    ))
                })?;

                Ok(sector)
            };

            cursor.need(30)?;
            let x_offset = cursor.get_i16_le();
            let y_offset = cursor.get_i16_le();
            let upper_texture = texture_name(cursor.get_name(), "upper")?;
            let lower_texture = texture_name(cursor.get_name(), "lower")?;
            let middle_texture = texture_name(cursor.get_name(), "middle")?;
            let sector = sector_number(cursor.get_u16_le())?;

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

    /// Looks up a sidedef number.
    pub fn get(&self, number: u16) -> Option<&Sidedef> {
        self.0.get(usize::from(number))
    }
}

impl Index<u16> for Sidedefs {
    type Output = Sidedef;

    /// Looks up a sidedef number.
    fn index(&self, number: u16) -> &Self::Output {
        &self.0[usize::from(number)]
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
/// [linedef]: crate::map::Linedef
/// [sector]: crate::map::Sector
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
    /// [sector]: crate::map::Sector
    pub upper_texture: Option<String>,

    /// Optional lower [texture] name, if the adjacent [sector]'s floor is higher.
    ///
    /// [texture]: crate::assets::Texture
    /// [sector]: crate::map::Sector
    pub lower_texture: Option<String>,

    /// Optional middle [texture] name. One-sided linedefs should always have a middle texture.
    /// Two-sided linedefs are usually transparent, though they sometimes have partially see-through
    /// textures such as for fences or windows.
    ///
    /// [texture]: crate::assets::Texture
    pub middle_texture: Option<String>,

    /// [Sector] number this sidedef faces or helps to surround.
    ///
    /// [sector]: crate::map::Sector
    pub sector: u16,
}

impl Sidedef {
    /// Looks up the sidedef's sector.
    pub fn sector<'map>(&self, map: &'map Map) -> &'map Sector {
        &map.sectors[self.sector]
    }
}
