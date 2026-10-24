/* gen2.c: what a generator's loop across `yield` costs when it holds an address into the heap contents
 * (what A14 rule 1 of phase3b-threads.md allows) against when it keeps a position and re-reads the
 * collection's header after each resume (this attack's section 6.1). Same workload as the threads group's
 * ../../bench/gen.c: one generator over the bytes of a 1 MiB String, run 100 times (104,857,600 yields), the
 * consumer sums them. Models 0, 1, 2 are gen.c's, copied unchanged in what they do.
 *
 *   -DMODEL=0  plain C: a loop over the bytes, no generator
 *   -DMODEL=1  gen.c model 1: the frame keeps an index and reads the String's header from the frame on each
 *              resume (`f->s.p[f->i]`, test `f->i < f->s.len`); next() not inlined
 *   -DMODEL=2  gen.c model 2: the same, next() inlined into the consumer's loop
 *   -DMODEL=5  the frame keeps a pointer and an end pointer into the bytes across `yield` (what rule 1 makes
 *              legal); next() inlined
 *   -DMODEL=6  the String lives in a class object's field and the generator walks it by position, re-reading
 *              the object's field (pointer and length) after each resume, as Python's list iterator does
 *              (the walk then stays safe whatever the consumer does to the field); next() inlined
 *   -DMODEL=7  as 6, next() not inlined
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n gen2.c -o bin/gen2_n.exe */
#include "../../../../redesign/bench/harness.h"
#include <string.h>

#ifndef MODEL
#define MODEL 0
#endif
#define LEN (1u << 20)
#define ROUNDS 100

typedef struct { uint8_t *p; size_t len, cap; } String;
static String make_text(size_t n, uint32_t seed) {
    String s = { (uint8_t *)malloc(n), n, n };
    if (!s.p) abort();
    for (size_t i = 0; i < n; i++) s.p[i] = (uint8_t)('a' + (i * 7 + seed) % 26);
    return s;
}
typedef struct { uint32_t count, type_index; String text; } Doc;   /* a class object with a String field */

#if MODEL == 1 || MODEL == 7
#define NEXT_ATTR __declspec(noinline)
#else
#define NEXT_ATTR __forceinline
#endif

#if MODEL == 1 || MODEL == 2
typedef struct { uint32_t state; uint32_t seed; size_t n; String s; size_t i; } Frame;
static NEXT_ATTR int letters_next(Frame *f, uint8_t *out) {
    switch (f->state) {
    case 0: f->s = make_text(f->n, f->seed); f->i = 0; /* fall through */
    case 1:
        if (f->i < f->s.len) { *out = f->s.p[f->i++]; f->state = 1; return 1; }
        free(f->s.p); f->state = 2; /* fall through */
    default: return 0;
    }
}
#elif MODEL == 5
typedef struct { uint32_t state; uint32_t seed; size_t n; String s; const uint8_t *cur, *end; } Frame;
static NEXT_ATTR int letters_next(Frame *f, uint8_t *out) {
    switch (f->state) {
    case 0: f->s = make_text(f->n, f->seed); f->cur = f->s.p; f->end = f->s.p + f->s.len; /* fall through */
    case 1:
        if (f->cur != f->end) { *out = *f->cur++; f->state = 1; return 1; }
        free(f->s.p); f->state = 2; /* fall through */
    default: return 0;
    }
}
#elif MODEL == 6 || MODEL == 7
typedef struct { uint32_t state; Doc *self; size_t i; } Frame;     /* the frame retains `self` ([CORO-12]) */
static NEXT_ATTR int letters_next(Frame *f, uint8_t *out) {
    switch (f->state) {
    case 0: f->i = 0; /* fall through */
    case 1: {
        String *s = &f->self->text;                 /* re-read the field after every resume */
        if (f->i < s->len) { *out = s->p[f->i++]; f->state = 1; return 1; }
        f->state = 2; /* fall through */
    }
    default: return 0;
    }
}
#endif

int main(void) {
    h_start();
    uint64_t sum = 0;
    for (int r = 0; r < ROUNDS; r++) {
#if MODEL == 0
        String s = make_text(LEN, (uint32_t)r);
        for (size_t i = 0; i < s.len; i++) sum += s.p[i];
        free(s.p);
#elif MODEL == 6 || MODEL == 7
        Doc *d = (Doc *)malloc(sizeof(Doc)); d->count = 1; d->type_index = 3; d->text = make_text(LEN, (uint32_t)r);
        Frame f = {0}; f.self = d;
        uint8_t c;
        while (letters_next(&f, &c)) sum += c;
        free(d->text.p); free(d);
#else
        Frame f = {0};
        f.seed = (uint32_t)r; f.n = LEN;
        uint8_t c;
        while (letters_next(&f, &c)) sum += c;
#endif
    }
    static const char *names[] = {"gen2-0-plainC", "gen2-1-position-call", "gen2-2-position-inlined", "", "",
                                  "gen2-5-pointer-inlined", "gen2-6-field-position-inlined", "gen2-7-field-position-call"};
    h_report(names[MODEL], sum);
    return 0;
}
