/* Ember's fix-it today for A25 ("store an owned copy: String for str"): every word is its own String,
   one heap allocation per word; element 24 bytes (ptr, len, cap: Ember's String layout). Python's
   str.split() also makes one object per word. */
#include "common.h"
typedef struct { char *p; size_t n, cap; } String;
int main(void) {
    h_start();
    size_t tn; char *text = make_text(&tn);
    uint64_t total = 0;
    for (int pass = 0; pass < PASSES; pass++) {
        char *low = lower(text, tn);
        String *w = NULL; size_t len = 0, cap = 0;
        for (size_t i = 0; i < tn;) {
            while (i < tn && low[i] == ' ') i++;
            size_t s = i;
            while (i < tn && low[i] != ' ') i++;
            if (i > s) {
                size_t n = i - s; char *c = malloc(n); memcpy(c, low + s, n);
                GROW(w, len, cap, String); w[len++] = (String){c, n, n};
            }
        }
        uint64_t x = 0;
        for (size_t k = 0; k < len; k++) x ^= fnv(w[k].p, w[k].n);
        total += x + len;
        for (size_t k = 0; k < len; k++) free(w[k].p);
        free(w); free(low);
    }
    h_report("a25-owned-malloc", total);
    return 0;
}
