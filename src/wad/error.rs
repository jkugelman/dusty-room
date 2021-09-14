use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::wad::{self, WadType};

#[derive(Error, Debug)]
pub enum Error {
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },

    #[error("{}: not {}", path.display(), match expected {
        WadType::Iwad => "an IWAD",
        WadType::Pwad => "a PWAD",
    })]
    WrongType { path: PathBuf, expected: WadType },

    #[error("{}: {desc}", path.display())]
    Malformed { path: PathBuf, desc: String },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn malformed(path: impl AsRef<Path>, desc: String) -> Self {
        Self::Malformed {
            path: path.as_ref().into(),
            desc,
        }
    }
}

/// Import this trait to add an extension methods to convert a [`std::io::Result`] into a
/// [`Result`].
pub trait ResultExt<T> {
    fn err_path(self, path: impl AsRef<Path>) -> wad::Result<T>;
}

impl<T> ResultExt<T> for io::Result<T> {
    /// Maps a [`std::io::Error`] into an [`Error::Io`] by adding a file path for
    /// context.
    fn err_path(self, path: impl AsRef<Path>) -> wad::Result<T> {
        self.map_err(|err| wad::Error::Io {
            path: path.as_ref().into(),
            source: err,
        })
    }
}
