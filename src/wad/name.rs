use std::borrow::Borrow;
use std::convert::TryInto;
use std::io::{self, BufRead, Cursor};
use std::slice;

/// Reads a name from a raw 8-byte, NUL padded byte array.
///
/// This function strips trailing NULs and verifies that all characters are ASCII.
///
/// # Errors
///
/// If the name contains any non-ASCII characters it is still converted into a string but is
/// returned as an `Err` instead for easier printing of the bad name.
pub fn parse_name(raw: &[u8; 8]) -> Result<&str, String> {
    let nul_index = raw.iter().position(|&ch| ch == b'\0').unwrap_or(raw.len());
    let slice = &raw[..nul_index];

    if raw.is_ascii() {
        Ok(std::str::from_utf8(slice).unwrap())
    } else {
        // Convert the name into a string. It might not be valid UTF-8 so don't bother with
        // `std::str::from_utf8`. Instead treat it as Latin-1, where all bytes are valid and map
        // 1-to-1 to the corresponding Unicode codepoints.
        Err(slice.iter().map(|&b| b as char).collect::<String>())
    }
}

pub fn read_name<'buf>(cursor: &mut Cursor<&'buf [u8]>) -> io::Result<Result<&'buf str, String>> {
    let buffer = cursor.fill_buf().unwrap();
    let name = buffer.get(..8).ok_or(io::ErrorKind::UnexpectedEof)?;
    // SAFETY: We know the cursor's underlying buffer has a lifetime of `'buf`, it's just `fill_buf`
    // doesn't preserve that information. It's safe to restore it.
    let name: &[u8] = unsafe { slice::from_raw_parts(name.as_ptr(), name.len()) };
    cursor.consume(8);

    Ok(parse_name(name.try_into().unwrap()))
}

pub trait NameExt {
    /// Returns `true` if this is a legal lump name consisting only of the letters `A-Z`, digits
    /// `0-9`, and any of the punctuation `[]-_\`.
    fn is_legal(&self) -> bool;
}

impl<S: Borrow<str>> NameExt for S {
    /// Returns `true` if this is a legal lump name consisting only of the letters `A-Z`, digits
    /// `0-9`, and any of the punctuation `[]-_\`.
    fn is_legal(&self) -> bool {
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
        assert!("PLAYPAL".is_legal());
        assert!("E1M8".is_legal());
        assert!("F1_101".is_legal());
        assert!("F-[_]\\R".is_legal());

        assert!(!"".is_legal());
        assert!(!"w104_1".is_legal());
        assert!(!"TOO_DARN_LONG".is_legal());
    }
}
