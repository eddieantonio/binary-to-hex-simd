//! The basic byte-by-byte way to convert bytes to hexadecimal.

pub fn to_ascii_hex(input: &[u8]) -> String {
    // The actual implementation (below) returns a `Vec<u8>`, but it's guaranteed to be valid ASCII
    // (and therefore UTF-8) bytes. Just convert it:
    unsafe { String::from_utf8_unchecked(_byte_by_byte(input)) }
}

/// This is exported so that other implementations can use it to convert the first few bytes of
/// input.
pub(super) fn _byte_by_byte(input: &[u8]) -> Vec<u8> {
    let mut vec = Vec::with_capacity(input.len() * 2);

    let to_hex_digit = |nibble| {
        if nibble >= 10 {
            b'A' + nibble - 10
        } else {
            b'0' + nibble
        }
    };

    for byte in input.iter().copied() {
        let high_nibble = byte >> 4;
        let low_nibble = byte & 0xF;

        vec.push(to_hex_digit(high_nibble));
        vec.push(to_hex_digit(low_nibble));
    }

    vec
}
