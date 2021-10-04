//! Load WAD files into memory and read their data lumps.
//!
//! # Examples
//!
//! Load a custom level:
//!
//! ```no_run
//! use kdoom::wad::Wad;
//!
//! let game_wad = Wad::load("doom.wad")?;
//! let my_wad = game_wad.patch("killer.wad")?;
//!
//! let my_level = my_wad.lumps_following("E1M1", 11)?;
//! let things_lump = my_level[1].expect_name("THINGS")?;
//! let sectors_lump = my_level[8].expect_name("SECTORS")?;
//! #
//! # Ok::<(), kdoom::wad::Error>(())
//! ```

pub use cursor::*;
pub use error::*;
pub use file::*;
pub use lump::*;
pub use name::*;
pub use wad::*;

#[cfg(test)]
pub(crate) mod test;

mod cursor;
mod error;
mod file;
mod lump;
mod name;
#[allow(clippy::module_inception)]
mod wad;
