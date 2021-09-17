use std::collections::BTreeMap;

use super::wad::{self, Wad};

#[derive(Clone, Debug)]
pub struct TextureBank {
    textures: BTreeMap<String, Texture>,
}

impl TextureBank {
    pub fn load(_wad: &Wad) -> wad::Result<Self> {
        Ok(Self {
            textures: BTreeMap::new(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Texture {}
