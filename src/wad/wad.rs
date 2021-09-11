use indexmap::IndexMap;
use std::sync::Arc;

pub trait Wad {
    /// Retrieves a named lump. The name must be unique.
    fn lump(&self, name: &str) -> Option<Arc<[u8]>>;

    /// Retrieves a block of `size` lumps following a named marker. The marker lump
    /// is not included in the result.
    fn lumps_after(&self, start: &str, size: usize) -> Option<LumpBlock>;

    /// Retrieves a block of lumps between start and end markers. The marker lumps
    /// are not included in the result.
    fn lumps_between(&self, start: &str, end: &str) -> Option<LumpBlock>;
}

pub type LumpBlock = IndexMap<String, Arc<[u8]>>;
