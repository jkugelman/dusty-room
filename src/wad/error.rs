use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::wad::{self, WadType};

/// A specialized [`Result`] type for [`Wad`] and [`WadFile`] operations. This typedef is used to
/// avoid writing out [`wad::Error`] directly and is otherwise a direct mapping to [`Result`].
///
/// [`Result`]: std::result::Result
/// [`Wad`]: crate::wad::Wad
/// [`WadFile`]: crate::wad::WadFile
pub type Result<T> = std::result::Result<T, Error>;

/// The error type when loading and searching [`Wad`]s and [`WadFile`]s. Errors are always tied to a
/// particular WAD file.
///
/// [`Wad`]: crate::wad::Wad
/// [`WadFile`]: crate::wad::WadFile
#[derive(Error, Debug)]
pub enum Error {
    /// An I/O error from a [`std::io`] operation.
    #[error("{}: {source}", path.display())]
    Io {
        /// The path of the file where the I/O error occurred.
        path: PathBuf,
        /// The source I/O error.
        source: io::Error,
    },

    /// An IWAD was received when expecting a PWAD, or vice versa.
    #[error("{}: not {}", path.display(), match expected {
        WadType::Iwad => "an IWAD",
        WadType::Pwad => "a PWAD",
    })]
    WrongType {
        /// The file path.
        path: PathBuf,
        /// The WAD type that was expected.
        expected: WadType,
    },

    /// A WAD file or set of WAD files is malformed or missing data.
    #[error("{}: {desc}", path.display())]
    Malformed {
        /// The path of the malformed file.
        path: PathBuf,
        /// A description of the error.
        desc: String,
    },
}

impl Error {
    /// Convenience method to create an [`Error::Malformed`].
    pub fn malformed(path: impl AsRef<Path>, desc: String) -> Self {
        Self::Malformed {
            path: path.as_ref().into(),
            desc,
        }
    }
}

/// Import this trait to add an extension methods to convert a [`std::io::Result`] into a
/// [`wad::Result`].
pub trait ResultExt<T> {
    /// Maps a [`std::io::Error`] into a [`wad::Error::Io`] by adding a file path for context.
    fn err_path(self, path: impl AsRef<Path>) -> wad::Result<T>;
}

impl<T> ResultExt<T> for io::Result<T> {
    /// Maps a [`std::io::Error`] into a [`wad::Error::Io`] by adding a file path for context.
    fn err_path(self, path: impl AsRef<Path>) -> wad::Result<T> {
        self.map_err(|err| wad::Error::Io {
            path: path.as_ref().into(),
            source: err,
        })
    }
}
