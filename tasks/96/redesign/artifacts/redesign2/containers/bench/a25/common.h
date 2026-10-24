/* A25 workload: `words = text.lower().split()`, then use every word, then drop the list.
   The text is 32 MiB of ASCII words (1 to 12 letters, some upper case) separated by single spaces,
   generated once from the shared generator (the whole process is timed; generation is the same in
   every variant). 5 passes; each pass lowers the whole text into a new buffer, splits it into a list
   of words in the variant's representation, hashes every word (FNV-1a) and counts them, then drops
   the list and the lowered text.
   Checksum: the xor of all word hashes plus the word count, summed over the passes. */
#include "../harness.h"
#include <string.h>
#define TEXT_BYTES (32u << 20)
#define PASSES 5
static char *make_text(size_t *len) {
    char *t = malloc(TEXT_BYTES);
    size_t n = 0;
    h_seed(11);
    while (n + 14 < TEXT_BYTES) {
        uint32_t wl = 1 + h_rand() % 12;
        for (uint32_t k = 0; k < wl; k++) {
            uint32_t r = h_rand();
            char c = (char)('a' + r % 26);
            if ((r >> 8) % 8 == 0) c = (char)(c - 'a' + 'A');
            t[n++] = c;
        }
        t[n++] = ' ';
    }
    *len = n;
    return t;
}
static char *lower(const char *t, size_t n) {            /* text.lower(): a new String */
    char *l = malloc(n);
    for (size_t i = 0; i < n; i++) { char c = t[i]; l[i] = (c >= 'A' && c <= 'Z') ? (char)(c + 32) : c; }
    return l;
}
static inline uint64_t fnv(const char *p, size_t n) {
    uint64_t h = 1469598103934665603ull;
    for (size_t i = 0; i < n; i++) { h ^= (unsigned char)p[i]; h *= 1099511628211ull; }
    return h;
}
#define GROW(arr, len, cap, T) do { if ((len) == (cap)) { (cap) = (cap) ? (cap) * 2 : 4; (arr) = realloc((arr), (cap) * sizeof(T)); } } while (0)
