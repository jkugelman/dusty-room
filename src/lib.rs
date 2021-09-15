pub mod assets;

pub use assets::Assets;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
pub(crate) mod test;
