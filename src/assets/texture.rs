use std::collections::BTreeMap;
use std::marker::PhantomData;

use crate::wad::{self, Wad};

#[derive(Clone, Debug)]
pub struct TextureBank<'wad> {
    _textures: BTreeMap<&'wad str, Texture<'wad>>,
}

impl<'wad> TextureBank<'wad> {
    pub fn load(_wad: &'wad Wad) -> wad::Result<Self> {
        Ok(Self {
            _textures: BTreeMap::new(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Texture<'wad> {
    _unused: PhantomData<&'wad ()>,
}
