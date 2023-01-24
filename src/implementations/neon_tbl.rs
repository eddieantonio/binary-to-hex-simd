//! Uses NEON's `tbl.16b` instruction to lookup 16 ASCII digits at a time.

use std::arch::asm;

pub fn to_ascii_hex(input: &[u8]) -> String {
    // How many bytes to process in one loop iteration:
    const MOUTHFUL: usize = 16;

    let output_size = input.len() * 2;
    let mut buffer = Vec::<u8>::with_capacity(output_size);

    // Since the SIMD portion can only do multiples of 16, we need to convert the first (up to) 15
    // bytes one a time:
    let n_initial_bytes = convert_initial_prefix_byte_by_byte::<MOUTHFUL>(input, &mut buffer);

    // Prepare the pointers for usage with _convert_fast():
    let bytes_written = n_initial_bytes * 2;
    let remaining_input = &input[n_initial_bytes..];
    let buffer_ptr = buffer[bytes_written..].as_mut_ptr();

    unsafe {
        _convert_fast(remaining_input, buffer_ptr);
        buffer.set_len(output_size);
        String::from_utf8_unchecked(buffer)
    }
}

/// Use's AArch64 NEON's `tbl.16b` to convert 16 nibbles to ASCII digits at a time.
#[inline]
unsafe fn _convert_fast(remaining_input: &[u8], buffer: *mut u8) {
    // ASCII lookup table used by tbl.16b:
    const LOOKUP: [u8; 16] = *b"0123456789ABCDEF";

    if remaining_input.is_empty() {
        // The assembly below assumes there is at least one loop iteration to do, or else it will
        // crash.
        return;
    }

    assert_eq!(
        0,
        remaining_input.len() % 16,
        "data size must be multiple of 16 bytes"
    );

    // Uh oh!
    asm!(
        // Initialization: load required constants:
        //
        //  v7 ← b"0123456789ABCDEF" -- the lookup table
        //  v6 ← [0xF; 16]           -- mask to extract the lower nibble
        "ldr        q7, [{3}]",
        "movi.16b   v6, #15",

        // Load 16 bytes from the input and increment input pointer:
        //
        //  v0 ← [█,▉,▉,▉]
        "2:",
        "ldp        q0, q1, [{0}], #16",

        // Split the input into vectors for the high and low nibbles:
        //
        //  v2 ← [▀,▀,▀,▀]
        //  v3 ← [▄,▄,▄,▄]
        "ushr.16b   v2, v0, #4",
        "and.16b    v3, v0, v6",

        // Interleave -- get the vectors back into order after we split in two:
        //
        //  v4 ← [▀,▄,▀,▄]
        //  v5 ← [▀,▄,▀,▄]
        "zip1.16b   v4, v2, v3",
        "zip2.16b   v5, v2, v3",

        // Lookup ASCII:
        //
        //  v4 ← [C,A,F,E] (from previous v4, [▀,▄,▀,▄])
        //  v5 ← [B,A,B,E] (from previous v5, [▀,▄,▀,▄])
        "tbl.16b    v4, {{ v7 }}, v4",
        "tbl.16b    v5, {{ v7 }}, v5",

        // Store 32 bytes of output:
        "stp        q4, q5, [{1}], #32",

        // Repeat for next 16 bytes of input:
        "subs       {2}, {2}, #16",
        "b.ne       2b",

        in(reg) remaining_input.as_ptr(),
        in(reg) buffer,
        in(reg) remaining_input.len(),
        in(reg) &LOOKUP,
        clobber_abi("C"),
        options(nostack),
    );
}

/// Converts `0..N` bytes of input from binary to its ASCII hexadecimal representation.
///
/// It's intended to be called by one of the SIMD implementations to convert the first few bytes
/// until the input is a multiple that the the implementation can deal with it.
///
/// Returns the number of bytes of input consumed.
///
/// ## Post-conditions:
///
/// There will be a multiple of N bytes left in the buffer to convert or in math:
/// `(input.len() - n_initial_bytes) % N == 0`
#[inline]
fn convert_initial_prefix_byte_by_byte<const N: usize>(
    input: &[u8],
    buffer: &mut Vec<u8>,
) -> usize {
    let n_initial_bytes = input.len() % N;

    let to_hex_digit = |nibble| {
        if nibble >= 10 {
            b'A' + nibble - 10
        } else {
            b'0' + nibble
        }
    };

    for byte in input[..n_initial_bytes].iter().copied() {
        let high_nibble = byte >> 4;
        let low_nibble = byte & 0xF;

        buffer.push(to_hex_digit(high_nibble));
        buffer.push(to_hex_digit(low_nibble));
    }

    n_initial_bytes
}
