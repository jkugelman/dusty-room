//! Place test assets in a public crate-wide location so tests don't waste time
//! loading the same files over and over.

#[cfg(test)]
use crate::{WadFile, WadStack};

pub static DOOM_WAD_PATH: &str = "test/doom.wad";
pub static DOOM2_WAD_PATH: &str = "test/doom2.wad";
pub static KILLER_WAD_PATH: &str = "test/killer.wad";
pub static BIOTECH_WAD_PATH: &str = "test/biotech.wad";

lazy_static! {
    pub static ref DOOM_WAD_FILE: WadFile = WadFile::open(DOOM_WAD_PATH).unwrap();
    pub static ref DOOM2_WAD_FILE: WadFile = WadFile::open(DOOM2_WAD_PATH).unwrap();
    pub static ref KILLER_WAD_FILE: WadFile = WadFile::open(KILLER_WAD_PATH).unwrap();
    pub static ref BIOTECH_WAD_FILE: WadFile = WadFile::open(BIOTECH_WAD_PATH).unwrap();
}

lazy_static! {
    pub static ref DOOM_WAD: WadStack = WadStack::unchecked(&*DOOM_WAD_FILE);
    pub static ref DOOM2_WAD: WadStack = WadStack::unchecked(&*DOOM2_WAD_FILE);
    pub static ref KILLER_WAD: WadStack = DOOM_WAD.and_unchecked(&*KILLER_WAD_FILE);
    pub static ref BIOTECH_WAD: WadStack = DOOM2_WAD.and_unchecked(&*BIOTECH_WAD_FILE);
}
