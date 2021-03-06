use std::borrow::Cow;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::wad::WadKind;

/// A specialized [`Result`] type for [`Wad`] and [`WadFile`] operations. This typedef is used to
/// avoid writing out [`wad::Error`] directly and is otherwise a direct mapping to [`Result`].
///
/// [`Result`]: std::result::Result
/// [`Wad`]: crate::wad::Wad
/// [`WadFile`]: crate::wad::WadFile
/// [`wad::Error`]: crate::wad::Error
pub type Result<T> = std::result::Result<T, Error>;

/// The error type when loading and searching [`Wad`]s and [`WadFile`]s. Errors are always tied to a
/// particular file.
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

    /// A [`Wad`] or [`WadFile`] is malformed or missing data.
    ///
    /// [`Wad`]: crate::wad::Wad
    /// [`WadFile`]: crate::wad::WadFile
    #[error("{}: {desc}", path.display())]
    Malformed {
        /// The path of the malformed file.
        path: PathBuf,
        /// A description of the error.
        desc: Cow<'static, str>,
    },

    /// An [IWAD] was received when expecting a [PWAD], or vice versa.
    ///
    /// [IWAD]: WadKind::Iwad
    /// [PWAD]: WadKind::Pwad
    #[error("{}: not {}", path.display(), match expected {
        WadKind::Iwad => "an IWAD",
        WadKind::Pwad => "a PWAD",
    })]
    WrongType {
        /// The file path.
        path: PathBuf,
        /// The WAD kind that was expected.
        expected: WadKind,
    },
}

impl Error {
    /// Creates an [`Error::Malformed`]. Accepts both `&'static str` literals and owned `String`s.
    pub fn malformed(path: impl AsRef<Path>, desc: impl Into<Cow<'static, str>>) -> Self {
        Self::Malformed { path: path.as_ref().to_owned(), desc: desc.into() }
    }
}
