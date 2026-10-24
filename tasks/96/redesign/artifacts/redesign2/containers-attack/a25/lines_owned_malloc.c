/* The answer AS WRITTEN for the line loop (lines.h): views of many texts are refused, and the fix-it is
   an owned copy per word ("String for str"): one allocation per word; each lowered line is freed at the
   end of its iteration. Element 24 bytes (ptr, len, cap). */
#include "lines.h"
typedef struct { char *p; size_t n, cap; } String;
int main(void) {
    h_start();
    size_t tn; char *text = make_lines(&tn);
    uint64_t total = 0;
    for (int pass = 0; pass < PASSES; pass++) {
        String *w = NULL; size_t len = 0, cap = 0;
        FOR_LINES(text, tn, s, e, {
            char *low = lower(text + s, e - s);
            FOR_WORDS(low, e - s, a, b, {
                size_t n = b - a; char *c = malloc(n); memcpy(c, low + a, n);
                GROW(w, len, cap, String); w[len++] = (String){c, n, n};
            })
            free(low);
        })
        uint64_t x = 0;
        for (size_t k = 0; k < len; k++) x ^= fnv(w[k].p, w[k].n);
        total += x + len;
        for (size_t k = 0; k < len; k++) free(w[k].p);
        free(w);
    }
    h_report("a25-lines-owned-malloc", total);
    return 0;
}
