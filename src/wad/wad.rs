pub trait Wad: 'static {
    /// Retrieves a named lump. The name must be unique.
    fn lump(&self, name: &str) -> Option<&Lump>;

    /// Retrieves a block of `size` lumps following a named marker. The marker lump
    /// is not included in the result.
    fn lumps_after(&self, start: &str, size: usize) -> Option<&[Lump]>;

    /// Retrieves a block of lumps between start and end markers. The marker lumps
    /// are not included in the result.
    fn lumps_between(&self, start: &str, end: &str) -> Option<&[Lump]>;
}

pub struct Lump {
    pub name: String,
    pub data: Vec<u8>,
}

impl Lump {
    pub fn size(&self) -> usize {
        self.data.len()
    }
}
