/* d1rt.h: the runtime of "copy instead of share" (phase3-design-D1.md), copied unchanged from
   target/redesign/bench/D1/w1.c: an Array value is one pointer to a counted buffer (24-byte header:
   count, shift version, len, cap; the elements follow); every in-place change first tests
   count == 1 (copy-on-write); push tests count and capacity in one branch; growth doubles from 4. */
#pragma once
#include "../sh.h"
#include <string.h>

static _Noreturn void d1_panic(const char *msg) {
    fprintf(stderr, "panic: %s\n", msg);
    fflush(stderr);
    ExitProcess(101);
}
static void *xmalloc(size_t n) {
    void *p = malloc(n);
    if (!p) d1_panic("out of memory");
    return p;
}

#ifdef BUF32 /* experiment: a 32-byte header, so elements start 32-byte aligned and never straddle lines */
typedef struct Buf { uint32_t count; uint32_t shifts; uint64_t len; uint64_t cap; uint64_t pad_; } Buf;
#else
typedef struct Buf { uint32_t count; uint32_t shifts; uint64_t len; uint64_t cap; } Buf; /* 24 bytes */
#endif
static Buf EMPTY = {0x80000000u, 0, 0, 0}; /* the shared empty buffer: never unique, never freed */
#define EL(b, T) ((T *)((Buf *)(b) + 1))
typedef void (*RetainFn)(void *elems, uint64_t n);

/* Make *slot unique (copy-on-write) with room for `need` elements; doubling from 4 ([HEAP-2]). */
static Buf *buf_reserve_slow(Buf **slot, size_t esz, uint64_t need, RetainFn retain) {
    Buf *b = *slot;
    uint64_t cap = b->cap;
    if (need > cap) {
        uint64_t nc = cap < 4 ? 4 : cap * 2;
        while (nc < need) nc *= 2;
        cap = nc;
    }
    if (b->count == 1) {
        b = realloc(b, sizeof(Buf) + cap * esz);
        if (!b) d1_panic("out of memory");
        b->cap = cap;
    } else {
        Buf *nb = xmalloc(sizeof(Buf) + cap * esz);
        nb->count = 1; nb->shifts = 0; nb->len = b->len; nb->cap = cap;
        memcpy(nb + 1, b + 1, b->len * esz);
        if (retain) retain(nb + 1, b->len);
        if (b != &EMPTY) b->count--; /* still shared by someone else: never reaches 0 here */
        b = nb;
    }
    *slot = b;
    return b;
}
/* D1_PROVEN: the uniqueness facts of section 1.5 item 2 hold everywhere (the registry is a local that is
   never copied), so the compiler drops every uniqueness test; an experiment, to price the tests. */
#ifdef D1_PROVEN
#define D1_UNIQ(b) 1
#else
#define D1_UNIQ(b) ((b)->count == 1)
#endif
static inline Buf *buf_mut(Buf **slot, size_t esz, RetainFn retain) { /* before an in-place write */
    Buf *b = *slot;
    if (__builtin_expect(!D1_UNIQ(b), 0)) b = buf_reserve_slow(slot, esz, b->len, retain);
    return b;
}
#define PUSH(slot, T, val, retain)                                                                   \
    do {                                                                                             \
        Buf *b_ = *(slot);                                                                           \
        if (__builtin_expect(!D1_UNIQ(b_) || b_->len == b_->cap, 0))                               \
            b_ = buf_reserve_slow((slot), sizeof(T), b_->len + 1, (retain));                         \
        EL(b_, T)[b_->len++] = (val);                                                                \
    } while (0)

/* Array[u32].resize(n, fill) */
static Buf *buf_resize_u32(Buf **slot, uint64_t n, uint32_t fill) {
    Buf *b = *slot;
    if (!D1_UNIQ(b) || n > b->cap) b = buf_reserve_slow(slot, 4, n, NULL);
    for (uint64_t i = b->len; i < n; i++) EL(b, uint32_t)[i] = fill;
    if (n > b->len) b->len = n;
    return b;
}
