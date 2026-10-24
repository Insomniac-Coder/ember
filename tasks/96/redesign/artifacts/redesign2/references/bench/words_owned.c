/* words_owned.c: every word an owned copy (what E3063's help asks for today, and what Python's
   split() does): a String header (pointer, length, capacity) and one allocation per word. */
#include "words_common.h"
typedef struct { char *p; size_t len, cap; } Str;
typedef struct { Str *a; size_t len, cap; } Words;
static inline void push(Words *w, Str s) {
    if (w->len == w->cap) { w->cap = w->cap ? w->cap * 2 : 16; w->a = (Str *)realloc(w->a, w->cap * sizeof(Str)); }
    w->a[w->len++] = s;
}
int main(void) {
    h_start(); h_seed(42);
    size_t len; char *t = make_text_into((char *)malloc((size_t)NW * 13), &len);
    work_start();
    Words w = {0};
    size_t s = 0;
    for (size_t i = 0; i <= len; i++)
        if (i == len || t[i] == ' ') {
            size_t n = i - s; char *c = (char *)malloc(n); memcpy(c, t + s, n);
            push(&w, (Str){c, n, n}); s = i + 1;
        }
    uint64_t sum = 0;
    for (int pass = 0; pass < PASSES; pass++)
        for (size_t k = 0; k < w.len; k++) sum += fnv(w.a[k].p, w.a[k].len);
    for (uint32_t r = 0; r < NRAND; r++) {
        size_t j = h_rand() % w.len;
        sum += w.a[j].len + (uint8_t)w.a[j].p[0];
    }
    for (size_t k = 0; k < w.len; k++) free(w.a[k].p);
    free(w.a); free(t);
    work_end();
    h_report("words-owned", sum);
    return 0;
}
