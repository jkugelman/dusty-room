#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapError {
    /// Required lump is missing.
    LumpMissing(String),
}
