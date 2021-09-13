use thiserror::Error;

use crate::wad;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    WadError {
        #[from]
        source: wad::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
