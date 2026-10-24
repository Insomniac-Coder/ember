/* words_counted16.c: the counted word, saved as far as I could: a 16-byte word (pointer + u32 start +
   u32 length, so a buffer is at most 4 GiB), the retains of one loop coalesced into one add after the
   loop (the compiler sees every pushed word retain `b`), and the Array drop releasing runs of equal
   buffers with one decrement per run. */
#include "words_common.h"
typedef struct { size_t count; char data[]; } Buf;
typedef struct { Buf *b; uint32_t start; uint32_t len; } Word;
typedef struct { Word *a; size_t len, cap; } Words;
static inline void push(Words *w, Word s) {
    if (w->len == w->cap) { w->cap = w->cap ? w->cap * 2 : 16; w->a = (Word *)realloc(w->a, w->cap * sizeof(Word)); }
    w->a[w->len++] = s;
}
static inline void release_n(Buf *b, size_t n) { if ((b->count -= n) == 0) free(b); }
int main(void) {
    h_start(); h_seed(42);
    Buf *b = (Buf *)malloc(sizeof(Buf) + (size_t)NW * 13);
    b->count = 1;
    size_t len; char *t = make_text_into(b->data, &len);
    work_start();
    Words w = {0};
    size_t s = 0, made = 0;
    for (size_t i = 0; i <= len; i++)
        if (i == len || t[i] == ' ') { push(&w, (Word){b, (uint32_t)s, (uint32_t)(i - s)}); made++; s = i + 1; }
    b->count += made;                                         /* coalesced retains */
    uint64_t sum = 0;
    for (int pass = 0; pass < PASSES; pass++)
        for (size_t k = 0; k < w.len; k++) sum += fnv(w.a[k].b->data + w.a[k].start, w.a[k].len);
    for (uint32_t r = 0; r < NRAND; r++) {
        size_t j = h_rand() % w.len;
        sum += w.a[j].len + (uint8_t)w.a[j].b->data[w.a[j].start];
    }
    Buf *last = NULL; size_t run = 0;                          /* run-length releases */
    for (size_t k = 0; k < w.len; k++) {
        if (w.a[k].b == last) run++;
        else { if (last) release_n(last, run); last = w.a[k].b; run = 1; }
    }
    if (last) release_n(last, run);
    free(w.a);
    release_n(b, 1);
    work_end();
    h_report("words-counted16", sum);
    return 0;
}
