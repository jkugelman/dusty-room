mod wad;

pub use wad::*;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
pub(crate) mod test;
