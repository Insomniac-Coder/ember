/* Last night's D1 answer for A25: a word is (buffer, offset, length) sharing the lowered text's
   buffer, which carries a count; each stored word adds one, each dropped word takes one away, and the
   buffer is freed when the count reaches 0. Non-atomic counts (one thread). 16 bytes per word. */
#include "common.h"
typedef struct { uint64_t count; char bytes[]; } Buf;
typedef struct { Buf *b; uint32_t off, n; } Word;
static Buf *lower_counted(const char *t, size_t n) {     /* text.lower() into a counted buffer */
    Buf *b = malloc(sizeof(Buf) + n); b->count = 1;
    for (size_t i = 0; i < n; i++) { char c = t[i]; b->bytes[i] = (c >= 'A' && c <= 'Z') ? (char)(c + 32) : c; }
    return b;
}
int main(void) {
    h_start();
    size_t tn; char *text = make_text(&tn);
    uint64_t total = 0;
    for (int pass = 0; pass < PASSES; pass++) {
        Buf *b = lower_counted(text, tn);
        Word *w = NULL; size_t len = 0, cap = 0;
        for (size_t i = 0; i < tn;) {
            while (i < tn && b->bytes[i] == ' ') i++;
            size_t s = i;
            while (i < tn && b->bytes[i] != ' ') i++;
            if (i > s) { GROW(w, len, cap, Word); b->count++; w[len++] = (Word){b, (uint32_t)s, (uint32_t)(i - s)}; }
        }
        if (--b->count == 0) free(b);                          /* the temporary's own reference */
        uint64_t x = 0;
        for (size_t k = 0; k < len; k++) x ^= fnv(w[k].b->bytes + w[k].off, w[k].n);
        total += x + len;
        for (size_t k = 0; k < len; k++) if (--w[k].b->count == 0) free(w[k].b);
        free(w);
    }
    h_report("a25-shared-count", total);
    return 0;
}
