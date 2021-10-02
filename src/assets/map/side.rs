use std::io::Cursor;
use std::sync::Arc;

use crate::assets::Texture;
use crate::wad::{self, Lump};

#[derive(Debug)]
pub struct Side {
    x_offset: i16,
    y_offset: i16,
    upper_texture: Arc<Texture>,
    lower_texture: Arc<Texture>,
    middle_texture: Arc<Texture>,
}

impl Side {
    pub fn load(lump: &Lump) -> wad::Result<Vec<Self>> {
        assert_eq!(lump.name(), "SIDEDEFS");

        let mut sides = Vec::with_capacity(lump.size() / 30);
        let mut cursor = Cursor::new(lump.data());

        Ok(sides)
    }
}
