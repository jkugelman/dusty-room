#![no_main]
use libfuzzer_sys::fuzz_target;

extern crate kdoom;

use std::io::Write;

use tempfile::NamedTempFile;

use kdoom::assets::Assets;
use kdoom::assets::wad::Wad;

fuzz_target!(|data: &[u8]| {
    if let Ok(mut file) = NamedTempFile::new() {
        if let Ok(()) = file.as_file_mut().write_all(data) {
            if let Ok(wad) = Wad::open(file.path()) {
                let _ = Assets::load(&wad);
            }
        }
    }
});
