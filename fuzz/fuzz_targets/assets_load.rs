#![no_main]
use libfuzzer_sys::fuzz_target;

extern crate kdoom;

use std::error::Error;
use std::io::Write;

use tempfile::NamedTempFile;

use kdoom::assets::Assets;
use kdoom::wad::Wad;

fuzz_target!(|data: &[u8]| {
    let result: Result<(), Box<dyn Error>> = (|| {
        let mut file = NamedTempFile::new()?;
        file.as_file_mut().write_all(data)?;

        let wad = Wad::load(file.path())?;
        let _ = Assets::load(&wad)?;

        println!("{}: 👍 assets loaded", file.path().display());
        Ok(())
    })();

    if let Err(err) = result {
        println!("{}", err);
    }
});
