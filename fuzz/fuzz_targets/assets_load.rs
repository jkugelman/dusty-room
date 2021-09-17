#![no_main]
use libfuzzer_sys::fuzz_target;

extern crate kdoom;

use std::io::Write;

use tempfile::NamedTempFile;

use kdoom::assets::wad::Wad;
use kdoom::assets::Assets;

fuzz_target!(|data: &[u8]| {
    let mut file = match NamedTempFile::new() {
        Ok(file) => file,
        Err(err) => {
            dbg!(err);
            return;
        }
    };

    match file.as_file_mut().write_all(data) {
        Ok(()) => {}
        Err(err) => {
            dbg!(err);
            return;
        }
    }

    let wad = match Wad::open(file.path()) {
        Ok(wad) => wad,
        Err(err) => {
            dbg!(err);
            return;
        }
    };

    let _ = match Assets::load(&wad) {
        Ok(assets) => assets,
        Err(err) => {
            dbg!(err);
            return;
        }
    };
});
