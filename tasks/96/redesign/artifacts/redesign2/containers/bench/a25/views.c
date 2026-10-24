/* The answer for A25: a word is a view (pointer, length) into the lowered text, which lives as long
   as the list (the compiler extends the temporary). 16 bytes per word, no allocation per word.
   This is also what Rust's Vec<&str> is. */
#include "common.h"
typedef struct { const char *p; size_t n; } Str;
int main(void) {
    h_start();
    size_t tn; char *text = make_text(&tn);
    uint64_t total = 0;
    for (int pass = 0; pass < PASSES; pass++) {
        char *low = lower(text, tn);
        Str *w = NULL; size_t len = 0, cap = 0;
        for (size_t i = 0; i < tn;) {
            while (i < tn && low[i] == ' ') i++;
            size_t s = i;
            while (i < tn && low[i] != ' ') i++;
            if (i > s) { GROW(w, len, cap, Str); w[len++] = (Str){low + s, i - s}; }
        }
        uint64_t x = 0;
        for (size_t k = 0; k < len; k++) x ^= fnv(w[k].p, w[k].n);
        total += x + len;
        free(w); free(low);
    }
    h_report("a25-views", total);
    return 0;
}
