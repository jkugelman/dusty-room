use std::{fmt, sync::Arc};

use crate::Lump;
#[allow(unused_imports)]
use crate::{WadFile, WadStack};

/// A `Wad` allows for retrieval of lumps in either a single [`WadFile`] or in a
/// layered [`WadStack`] of files.
pub trait Wad: fmt::Debug {
    /// Retrieves a named lump. The name must be unique.
    fn lump(&self, name: &str) -> Option<&Lump>;

    /// Retrieves a block of `size` lumps starting with a named marker. The marker
    /// lump is included in the result.
    fn lumps_after(&self, start: &str, size: usize) -> Option<&[Lump]>;

    /// Retrieves a block of lumps between start and end markers. The marker lumps
    /// are included in the result.
    fn lumps_between(&self, start: &str, end: &str) -> Option<&[Lump]>;
}

/// Allows `&`[`WadFile`] references to be added to a [`WadStack`] so the stack
/// doesn't take ownership of the files. This could be useful to add the same
/// file to multiple stacks while loading it only once.
impl<W: Wad + ?Sized> Wad for &'_ W {
    fn lump(&self, name: &str) -> Option<&Lump> {
        (**self).lump(name)
    }

    fn lumps_after(&self, start: &str, size: usize) -> Option<&[Lump]> {
        (**self).lumps_after(start, size)
    }

    fn lumps_between(&self, start: &str, end: &str) -> Option<&[Lump]> {
        (**self).lumps_between(start, end)
    }
}

/// Allows `Box<dyn Wad>` to be added to a [`WadStack`], allowing for stacks
/// within stacks and other silliness.
impl<W: Wad + ?Sized> Wad for Box<W> {
    fn lump(&self, name: &str) -> Option<&Lump> {
        (**self).lump(name)
    }

    fn lumps_after(&self, start: &str, size: usize) -> Option<&[Lump]> {
        (**self).lumps_after(start, size)
    }

    fn lumps_between(&self, start: &str, end: &str) -> Option<&[Lump]> {
        (**self).lumps_between(start, end)
    }
}

/// Allows shared `Arc<dyn Wad>` and `Arc<WadFile>` references to be added to a
/// [`WadStack`] so the stack doesn't take ownership of the files. This could be
/// useful to add the same file to multiple stacks while loading it only once.
impl<W: Wad + ?Sized> Wad for Arc<W> {
    fn lump(&self, name: &str) -> Option<&Lump> {
        (**self).lump(name)
    }

    fn lumps_after(&self, start: &str, size: usize) -> Option<&[Lump]> {
        (**self).lumps_after(start, size)
    }

    fn lumps_between(&self, start: &str, end: &str) -> Option<&[Lump]> {
        (**self).lumps_between(start, end)
    }
}
