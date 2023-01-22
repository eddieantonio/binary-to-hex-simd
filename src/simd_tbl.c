#include <stdint.h>
#include <stdio.h>

/**
 * Converts the binary in src to an ASCII hexadecimal representation in src to
 * dest.
 *
 * The input is n bytes, and the destination must have room for at least 2 *
 * n bytes.
 */
void to_hex_using_tbl(uint8_t *src, uint8_t *dest, size_t n) {
    static char lookup[] = "0123456789ABCDEF";

    __asm__(
            /* Load required constants: */
            "ldr        q6,[%0]\n"
            "movi.16b   v5, #15\n"

            /* Load 16-bytes from the input and increment pointer: */
            "ldr        q0, [x0], #16\n"

            /* Split into high and low nibbles: */
            "ushr.16b   v1, v0, #4\n"
            "and.16b    v2, v0, v5\n"

            /* Interleave: */
            "zip1.16b   v3, v1, v2\n"
            "zip2.16b   v4, v1, v2\n"

            /* Lookup ASCII: */
            "tbl.16b    v3, { v6 }, v3\n"
            "tbl.16b    v4, { v6 }, v4\n"

            /* Store: */
            "stp        q3, q3, [x1], #32\n"

                : /* no outputs */
                : "p" (lookup)
            );
}



int main(void) {
    char src[] = "Hello!\n";
    uint8_t buffer[33] = {0};

    to_hex_using_tbl((uint8_t*) src, buffer, sizeof(src));
    buffer[2 * (sizeof(src) - 1) + 1] = 0;
    printf("<<%s>>\n", buffer);
    return 0;
}


void write_hex_slow(uint8_t *src, uint8_t *dest, size_t n) {
    while (n > 0) {
        uint8_t c = *src++;
        uint8_t hi = c >> 4;
        uint8_t lo = c & 0xf;

        *dest++ = hi < 10 ? hi + '0' : hi - 10 + 'A';
        *dest++ = lo < 10 ? lo + '0' : lo - 10 + 'A';
        n--;
    }
}
