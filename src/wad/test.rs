//! Place test WADs in a public crate-wide location so tests don't waste time loading the same
//! files over and over.

use crate::wad::Wad;

pub static DOOM_WAD_PATH: &str = "test/doom.wad";
pub static DOOM2_WAD_PATH: &str = "test/doom2.wad";
pub static KILLER_WAD_PATH: &str = "test/killer.wad";
pub static BIOTECH_WAD_PATH: &str = "test/biotech.wad";

lazy_static! {
    pub static ref DOOM_WAD: Wad = Wad::open(DOOM_WAD_PATH).unwrap();
    pub static ref DOOM2_WAD: Wad = Wad::open(DOOM2_WAD_PATH).unwrap();
    pub static ref KILLER_WAD: Wad = DOOM_WAD.patch(KILLER_WAD_PATH).unwrap();
    pub static ref BIOTECH_WAD: Wad = DOOM2_WAD.patch(BIOTECH_WAD_PATH).unwrap();
}
