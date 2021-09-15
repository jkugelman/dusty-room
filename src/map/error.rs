use thiserror::Error;

use crate::wad;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Wad {
        #[from]
        source: wad::Error,
    },
}