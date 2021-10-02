use std::borrow::Borrow;

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

pub trait NameExt {
    /// Returns `true` if this is a legal lump name consisting only of the letters `A-Z`, digits
    /// `0-9`, and any of the punctuation `[]-_\`.
    fn is_legal_name(&self) -> bool;
}

impl<S: Borrow<str>> NameExt for S {
    /// Returns `true` if this is a legal lump name consisting only of the letters `A-Z`, digits
    /// `0-9`, and any of the punctuation `[]-_\`.
    fn is_legal_name(&self) -> bool {
        let name = self.borrow();

        let good_length = !name.is_empty() && name.len() <= 8;
        let has_illegal_char =
            name.contains(|ch| !matches!(ch, 'A'..='Z' | '0'..='9' | '[' | ']' | '-' | '_' | '\\'));
        good_length && !has_illegal_char
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_legal() {
        assert!("PLAYPAL".is_legal_name());
        assert!("E1M8".is_legal_name());
        assert!("F1_101".is_legal_name());
        assert!("F-[_]\\R".is_legal_name());

        assert!(!"".is_legal_name());
        assert!(!"w104_1".is_legal_name());
        assert!(!"TOO_DARN_LONG".is_legal_name());
    }
}
