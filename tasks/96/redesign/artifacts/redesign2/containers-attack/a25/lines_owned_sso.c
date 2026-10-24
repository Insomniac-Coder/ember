/* The line loop (lines.h) with owned words in the short-string layout (24 bytes, up to 23 inline): the
   report's cheaper owned form (its owned_sso.c), here as the fix-it for refused views. */
#include "lines.h"
typedef struct { union { char in[23]; struct { char *p; size_t n; } h; } u; unsigned char tag; } SsoString;
static inline const char *sso_ptr(const SsoString *s) { return s->tag == 255 ? s->u.h.p : s->u.in; }
static inline size_t sso_len(const SsoString *s) { return s->tag == 255 ? s->u.h.n : s->tag; }
int main(void) {
    h_start();
    size_t tn; char *text = make_lines(&tn);
    uint64_t total = 0;
    for (int pass = 0; pass < PASSES; pass++) {
        SsoString *w = NULL; size_t len = 0, cap = 0;
        FOR_LINES(text, tn, s, e, {
            char *low = lower(text + s, e - s);
            FOR_WORDS(low, e - s, a, b, {
                size_t n = b - a; SsoString el;
                if (n <= 23) { memcpy(el.u.in, low + a, n); el.tag = (unsigned char)n; }
                else { el.u.h.p = malloc(n); memcpy(el.u.h.p, low + a, n); el.u.h.n = n; el.tag = 255; }
                GROW(w, len, cap, SsoString); w[len++] = el;
            })
            free(low);
        })
        uint64_t x = 0;
        for (size_t k = 0; k < len; k++) x ^= fnv(sso_ptr(&w[k]), sso_len(&w[k]));
        total += x + len;
        for (size_t k = 0; k < len; k++) if (w[k].tag == 255) free(w[k].u.h.p);
        free(w);
    }
    h_report("a25-lines-owned-sso", total);
    return 0;
}
