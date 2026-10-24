/* The SAVED answer for the line loop (lines.h): "a list takes its texts along". Each word is a view
   (pointer, length) into its lowered line; the lowered line, a temporary that would die at the end of
   its iteration, is moved into the list's owner block instead (one pointer pushed per line, not per
   word). The owner block is freed with the list. */
#include "lines.h"
typedef struct { const char *p; size_t n; } Str;
int main(void) {
    h_start();
    size_t tn; char *text = make_lines(&tn);
    uint64_t total = 0;
    for (int pass = 0; pass < PASSES; pass++) {
        Str *w = NULL; size_t len = 0, cap = 0;
        char **own = NULL; size_t on = 0, oc = 0;           /* the list's owner block */
        FOR_LINES(text, tn, s, e, {
            char *low = lower(text + s, e - s);
            FOR_WORDS(low, e - s, a, b, { GROW(w, len, cap, Str); w[len++] = (Str){low + a, b - a}; })
            GROW(own, on, oc, char *); own[on++] = low;      /* taken along, not freed */
        })
        uint64_t x = 0;
        for (size_t k = 0; k < len; k++) x ^= fnv(w[k].p, w[k].n);
        total += x + len;
        free(w);
        for (size_t k = 0; k < on; k++) free(own[k]);
        free(own);
    }
    h_report("a25-lines-takealong", total);
    return 0;
}
