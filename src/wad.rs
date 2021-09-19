//! Load WAD files into memory and read their data lumps.
//!
//! # Examples
//!
//! Load the game IWAD and add a custom level.
//!
//! ```
//! use kdoom::wad::Wad;
//!
//! let game_wad = Wad::load("doom.wad")?;
//! let my_wad = game.patch("killer.wad")?;
//!
//! let my_level = my_wad.lumps_following("E1M1", 11)?;
//! let things_lump = my_level.get_with_name(1, "THINGS")?;
//! let sectors_lump = my_level.get_with_name(8, "SECTORS")?;
//! ```

pub use error::*;
pub use file::*;
pub use lump::*;
pub use wad::*;

#[cfg(test)]
pub(crate) mod test;

mod error;
mod file;
mod lump;
#[allow(clippy::module_inception)]
mod wad;
