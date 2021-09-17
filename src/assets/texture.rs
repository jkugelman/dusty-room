use std::collections::BTreeMap;

use super::wad::{self, Wad};

#[derive(Clone, Debug)]
pub struct TextureBank {
    _textures: BTreeMap<String, Texture>,
}

impl TextureBank {
    pub fn load(_wad: &Wad) -> wad::Result<Self> {
        Ok(Self {
            _textures: BTreeMap::new(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Texture {}
