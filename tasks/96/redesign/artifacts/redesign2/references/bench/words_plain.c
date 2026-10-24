/* words_plain.c: plain C, and the C that the self-borrowing struct (A7), the extended temporary (A22)
   and a region-carrying Array[str] compile to: a word is (pointer, length) into the one text buffer. */
#include "words_common.h"
typedef struct { const char *p; size_t n; } Slice;                 /* Ember `str` */
typedef struct { Slice *a; size_t len, cap; } Words;               /* Ember `Array[str]` */
static inline void push(Words *w, Slice s) {
    if (w->len == w->cap) { w->cap = w->cap ? w->cap * 2 : 16; w->a = (Slice *)realloc(w->a, w->cap * sizeof(Slice)); }
    w->a[w->len++] = s;
}
int main(void) {
    h_start(); h_seed(42);
    size_t len; char *t = make_text_into((char *)malloc((size_t)NW * 13), &len);
    work_start();
    Words w = {0};
    size_t s = 0;
    for (size_t i = 0; i <= len; i++)
        if (i == len || t[i] == ' ') { push(&w, (Slice){t + s, i - s}); s = i + 1; }
    uint64_t sum = 0;
    for (int pass = 0; pass < PASSES; pass++)
        for (size_t k = 0; k < w.len; k++) sum += fnv(w.a[k].p, w.a[k].n);
    for (uint32_t r = 0; r < NRAND; r++) {
        size_t j = h_rand() % w.len;
        sum += w.a[j].n + (uint8_t)w.a[j].p[0];
    }
    free(w.a); free(t);
    work_end();
    h_report("words-plain", sum);
    return 0;
}
