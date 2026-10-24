/* words_counted24.c: a word keeps its text alive (phase 1 A7(b) and A22(b); phase3-design-D1's String):
   the buffer has a count; a word is (buffer, start, length), three 8-byte words, retained when made,
   released when dropped. Non-atomic counts (one thread). */
#include "words_common.h"
typedef struct { size_t count; char data[]; } Buf;
typedef struct { Buf *b; size_t start; size_t len; } Word;
typedef struct { Word *a; size_t len, cap; } Words;
static inline void push(Words *w, Word s) {
    if (w->len == w->cap) { w->cap = w->cap ? w->cap * 2 : 16; w->a = (Word *)realloc(w->a, w->cap * sizeof(Word)); }
    w->a[w->len++] = s;
}
static inline void release(Buf *b) { if (--b->count == 0) free(b); }
int main(void) {
    h_start(); h_seed(42);
    Buf *b = (Buf *)malloc(sizeof(Buf) + (size_t)NW * 13);
    b->count = 1;                                   /* the text's own owner */
    size_t len; char *t = make_text_into(b->data, &len);
    work_start();
    Words w = {0};
    size_t s = 0;
    for (size_t i = 0; i <= len; i++)
        if (i == len || t[i] == ' ') { b->count++; push(&w, (Word){b, s, i - s}); s = i + 1; }
    uint64_t sum = 0;
    for (int pass = 0; pass < PASSES; pass++)
        for (size_t k = 0; k < w.len; k++) sum += fnv(w.a[k].b->data + w.a[k].start, w.a[k].len);
    for (uint32_t r = 0; r < NRAND; r++) {
        size_t j = h_rand() % w.len;
        sum += w.a[j].len + (uint8_t)w.a[j].b->data[w.a[j].start];
    }
    for (size_t k = 0; k < w.len; k++) release(w.a[k].b);   /* Array[String] drop: one release per element */
    free(w.a);
    release(b);                                               /* the text's owner */
    work_end();
    h_report("words-counted24", sum);
    return 0;
}
