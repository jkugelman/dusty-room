pub use assets::*;
pub use flat::*;
pub use geom::*;
pub use map::*;
pub use palette::*;
pub use patch::*;
pub use texture::*;

pub(self) use error::*;

#[allow(clippy::module_inception)]
mod assets;
mod error;
mod flat;
mod geom;
mod map;
mod palette;
mod patch;
mod texture;
