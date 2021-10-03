/// Reads a name from a raw 8-byte, NUL padded byte array.
///
/// This function does not check if the name contains only legal ASCII characters. Non-ASCII bytes
/// are treated as Latin-1, where all bytes are valid and map 1-to-1 to the corresponding Unicode
/// codepoints.
pub fn parse_name(raw: &[u8; 8]) -> String {
    let nul_index = raw.iter().position(|&ch| ch == b'\0').unwrap_or(raw.len());
    let raw = &raw[..nul_index];
    raw.iter().copied().map(|b| b as char).collect()
}
