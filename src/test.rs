//! Place test assets in a public crate-wide location so tests don't waste time
//! loading the same files over and over.
use std::sync::Arc;

use crate::wad::{Wad, WadFile};

pub static DOOM_WAD_PATH: &str = "test/doom.wad";
pub static DOOM2_WAD_PATH: &str = "test/doom2.wad";
pub static KILLER_WAD_PATH: &str = "test/killer.wad";
pub static BIOTECH_WAD_PATH: &str = "test/biotech.wad";

lazy_static! {
    pub static ref DOOM_WAD_FILE: Arc<WadFile> = Arc::new(WadFile::open(DOOM_WAD_PATH).unwrap());
    pub static ref DOOM2_WAD_FILE: Arc<WadFile> = Arc::new(WadFile::open(DOOM2_WAD_PATH).unwrap());
    pub static ref KILLER_WAD_FILE: Arc<WadFile> =
        Arc::new(WadFile::open(KILLER_WAD_PATH).unwrap());
    pub static ref BIOTECH_WAD_FILE: Arc<WadFile> =
        Arc::new(WadFile::open(BIOTECH_WAD_PATH).unwrap());
}

lazy_static! {
    pub static ref DOOM_WAD: Wad = Wad::initial(DOOM_WAD_FILE.clone()).unwrap();
    pub static ref DOOM2_WAD: Wad = Wad::initial(DOOM2_WAD_FILE.clone()).unwrap();
    pub static ref KILLER_WAD: Wad = DOOM_WAD.patch(KILLER_WAD_FILE.clone()).unwrap();
    pub static ref BIOTECH_WAD: Wad = DOOM2_WAD.patch(BIOTECH_WAD_FILE.clone()).unwrap();
}
