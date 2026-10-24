/* words_ember.c: words_plain.c (../../bench) with the two checks the Ember program pays that plain C does not:
   `words[j]` is bounds-checked and `w[0]` is bounds-checked on the word. Everything else is words_plain.c.
   This is what A7's Document (text and words in one value) costs in Ember, and also what the same program
   with the text held outside costs: A7 changes nothing in the emitted code. */
#include "words_common.h"
typedef struct { const char *p; size_t n; } Slice;                 /* Ember `str` */
typedef struct { Slice *a; size_t len, cap; } Words;               /* Ember `Array[str]` */
static void panic_(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
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
        if (j >= w.len) panic_("index out of range");          /* words[j] */
        Slice x = w.a[j];
        if (x.n == 0) panic_("index out of range");            /* w[0] */
        sum += x.n + (uint8_t)x.p[0];
    }
    free(w.a); free(t);
    work_end();
    h_report("words-ember-model", sum);
    return 0;
}
