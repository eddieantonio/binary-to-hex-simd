#include <stdint.h>
#include <stdio.h>

static void write_hex_slow(uint8_t *src, uint8_t *dest, size_t n) {
    while (n > 0) {
        uint8_t c = *src++;
        uint8_t hi = c >> 4;
        uint8_t lo = c & 0xf;

        *dest++ = hi < 10 ? hi + '0' : hi - 10 + 'A';
        *dest++ = lo < 10 ? lo + '0' : lo - 10 + 'A';
        n--;
    }
}

void to_hex_using_tbl32(uint8_t *src, uint8_t *dest, size_t n) {
    static char lookup[] = "0123456789ABCDEF";

    /* Start by writing bytes that don't fit in exactly 16-byte units: */
    size_t n_initial_bytes = n % 32;
    write_hex_slow(src, dest, n_initial_bytes);

    n -= n_initial_bytes;
    if (n == 0)
        /* Nothing left to do: */
        return;

    /* Skip the bytes we already (slowly) wrote. */
    src += n_initial_bytes;
    dest += 2 * n_initial_bytes;

    __asm__(
            /* Load required constants: */
            "ldr        q7, [%3]\n"
            "movi.16b   v6, #15\n"

            /* Offset destination by 32 bytes to facilitate these
             * instructions:
             *  stp  x,y, [reg, -#32]
             *  stp  x,y [reg], #64
             */
            "add       %1, %1, #32\n"

            /* Load 32-bytes from the input and increment pointer: */
            "L1:\n"
            "ldp        q0, q1, [%0], #32\n"

            /* Split into high and low nibbles: */
            "ushr.16b   v2, v0, #4\n"
            "and.16b    v3, v0, v6\n"

            /* Interleave: */
            "zip1.16b   v4, v2, v3\n"
            "zip2.16b   v5, v2, v3\n"

            /* Lookup ASCII: */
            "tbl.16b    v4, { v7 }, v4\n"
            "tbl.16b    v5, { v7 }, v5\n"

            /* Store 32 bytes of output */
            "stp        q4, q5, [%1, #-32]\n"

            /* Rinse and repeat for upper half */
            "ushr.16b   v2, v1, #4\n"
            "and.16b    v3, v1, v6\n"
            "zip1.16b   v4, v2, v3\n"
            "zip2.16b   v5, v2, v3\n"
            "tbl.16b    v4, { v7 }, v4\n"
            "tbl.16b    v5, { v7 }, v5\n"
            "stp        q4, q5, [%1], #64\n"

            "subs       %2, %2, #32\n"
            "b.ne       L1\n"

                : /* no outputs */
                : "p" (src),
                  "p" (dest),
                  "r" (n),
                  "p" (lookup)
            );
}

/**
 * Converts the binary in src to an ASCII hexadecimal representation in src to
 * dest.
 *
 * The input is n bytes, and the destination must have room for at least 2 *
 * n bytes.
 */
void to_hex_using_tbl(uint8_t *src, uint8_t *dest, size_t n) {
    static char lookup[] = "0123456789ABCDEF";

    /* Start by writing bytes that don't fit in exactly 16-byte units: */
    size_t n_initial_bytes = n % 16;
    write_hex_slow(src, dest, n_initial_bytes);

    n -= n_initial_bytes;
    if (n == 0)
        /* Nothing left to do: */
        return;

    /* Skip the bytes we already (slowly) wrote. */
    src += n_initial_bytes;
    dest += 2 * n_initial_bytes;

    __asm__(
            /* Load required constants: */
            "ldr        q7, [%3]\n"
            "movi.16b   v6, #15\n"

            /* Load 16-bytes from the input and increment pointer: */
            "L0:\n"
            "ldr        q0, [%0], #16\n"

            /* Split into high and low nibbles: */
            "ushr.16b   v1, v0, #4\n"
            "and.16b    v2, v0, v6\n"

            /* Interleave: */
            "zip1.16b   v3, v1, v2\n"
            "zip2.16b   v4, v1, v2\n"

            /* Lookup ASCII: */
            "tbl.16b    v3, { v7 }, v3\n"
            "tbl.16b    v4, { v7 }, v4\n"

            /* Store 32 bytes of output */
            "stp        q3, q4, [%1], #32\n"

            "subs       %2, %2, #16\n"
            "b.ne       L0\n"

                : /* no outputs */
                : "p" (src),
                  "p" (dest),
                  "r" (n),
                  "p" (lookup)
            );
}
