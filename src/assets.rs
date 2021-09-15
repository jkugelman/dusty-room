pub mod map;
pub mod palette;
pub mod wad;

pub use palette::{Palette, Palettes};
pub use wad::Wad;

pub struct Assets {
    _wad: Wad,
    _palette: Palette,
}
