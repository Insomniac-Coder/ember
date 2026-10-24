/* A candidate kind (c) answer for A25: owned words with the short-string layout, a 24-byte String
   that holds up to 23 bytes inline (no allocation) and points to the heap only for longer text. Every
   word here is at most 12 bytes, so no word allocates. Ember's String has no such layout today (it is
   the runtime's ember_vec: ptr, len, cap). */
#include "common.h"
typedef struct { union { char in[23]; struct { char *p; size_t n; } h; } u; unsigned char tag; } SsoString; /* tag: inline length, or 255 = heap */
static inline const char *sso_ptr(const SsoString *s) { return s->tag == 255 ? s->u.h.p : s->u.in; }
static inline size_t sso_len(const SsoString *s) { return s->tag == 255 ? s->u.h.n : s->tag; }
int main(void) {
    h_start();
    size_t tn; char *text = make_text(&tn);
    uint64_t total = 0;
    for (int pass = 0; pass < PASSES; pass++) {
        char *low = lower(text, tn);
        SsoString *w = NULL; size_t len = 0, cap = 0;
        for (size_t i = 0; i < tn;) {
            while (i < tn && low[i] == ' ') i++;
            size_t s = i;
            while (i < tn && low[i] != ' ') i++;
            if (i > s) {
                size_t n = i - s; SsoString e;
                if (n <= 23) { memcpy(e.u.in, low + s, n); e.tag = (unsigned char)n; }
                else { e.u.h.p = malloc(n); memcpy(e.u.h.p, low + s, n); e.u.h.n = n; e.tag = 255; }
                GROW(w, len, cap, SsoString); w[len++] = e;
            }
        }
        uint64_t x = 0;
        for (size_t k = 0; k < len; k++) x ^= fnv(sso_ptr(&w[k]), sso_len(&w[k]));
        total += x + len;
        for (size_t k = 0; k < len; k++) if (w[k].tag == 255) free(w[k].u.h.p);
        free(w); free(low);
    }
    h_report("a25-owned-sso", total);
    return 0;
}
