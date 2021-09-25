use std::io;

use crate::wad;

/// `LoadError` simplifies error handling in asset loading code.
///
/// A lot of the asset loading code uses [`Cursor`]s to parse byte slices. `LoadError` reduces the
/// hassle of catching all of the [`io::Error`]s. They're not actual I/O errors--they're all just
/// `UnexpectedEof`--so `LoadError` converts them all to `BadLump`. The loaders can then call
/// [`explain`] from one place to generate a `[wad::Error]` with a generic error message like "bad
/// lump data" or whatever.
///
/// Loading code has the option to generate full `wad::Error`s if it wants to return a more
/// informative error message.
///
/// [`Cursor`]: io::Cursor
/// [`explain`]: ResultExt::explain
#[derive(Debug)]
pub(super) enum LoadError {
    BadLump,
    Wad(wad::Error),
}

/// Import this trait to add an extension method to convert a `Result<_, LoadError>` into a
/// [`wad::Result`].
pub(super) trait ResultExt<T> {
    /// Maps a [`LoadError::OutOfBounds`] into a [`wad::Error`].
    fn explain(self, desc: impl FnOnce() -> wad::Error) -> wad::Result<T>;
}

impl<T> ResultExt<T> for Result<T, LoadError> {
    fn explain(self, desc: impl FnOnce() -> wad::Error) -> wad::Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(LoadError::BadLump) => Err(desc()),
            Err(LoadError::Wad(err)) => Err(err),
        }
    }
}

impl From<io::Error> for LoadError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::UnexpectedEof => Self::BadLump,
            _ => {
                panic!("std::io::Cursor returned unexpected {}", err);
            }
        }
    }
}

impl From<wad::Error> for LoadError {
    fn from(err: wad::Error) -> Self {
        Self::Wad(err)
    }
}
