/* m1rt.h: the small runtime every M1 C program shares (agent M1, spec 0.9.15_Hardened_1).
   Portable to clang -O2 and MSVC cl /O2 on Windows x64: no statement expressions, no VLAs; the
   atomics are the <intrin.h> Interlocked intrinsics both compilers have. On x64 every locked
   read-modify-write is a full barrier, so a "relaxed increment" and an "acquire-release decrement"
   ([RT-8]) are the same `lock xadd` instruction the C11 forms compile to. */
#pragma once
#include "harness.h"
#include <string.h>
#include <intrin.h>

#define NOINLINE __declspec(noinline)
#define NORETURN __declspec(noreturn)
#if defined(__clang__)
#define UNLIKELY(c) __builtin_expect(!!(c), 0)
#else
#define UNLIKELY(c) (c)
#endif

/* A stop: Ember's panic path ([RT-7]'s text), out of line, never returns. */
static NOINLINE NORETURN void e_stop(const char *msg) {
    fflush(stdout);
    fprintf(stderr, "panic: %s\n", msg);
    exit(101);
}

/* Atomics on a 32-bit or 64-bit count (each returns the value before the add). */
#define AT_ADD32(p, v) ((uint32_t)_InterlockedExchangeAdd((volatile long *)(p), (long)(v)))
#define AT_CAS32(p, expect, want) ((uint32_t)_InterlockedCompareExchange((volatile long *)(p), (long)(want), (long)(expect)))
#define AT_ADD64(p, v) ((uint64_t)_InterlockedExchangeAdd64((volatile long long *)(p), (long long)(v)))
/* An acquire load: on x64 a plain load is acquire in hardware; `volatile` stops the compiler moving it. */
#define AT_LOAD32(p) (*(volatile uint32_t *)(p))

/* The slab for one object size (run 3's ember_rt.h allocator): chunks of 4,096 blocks carved in
   order, then a free list. Class objects and function-value states come from it. */
typedef struct { char *cur, *end; void *free; size_t size; } Slab;
static inline void *slab_alloc(Slab *s) {
    if (s->free) { void *p = s->free; s->free = *(void **)p; return p; }
    if (s->cur == s->end) {
        s->cur = (char *)malloc(s->size * 4096);
        if (!s->cur) e_stop("out of memory");
        s->end = s->cur + s->size * 4096;
    }
    void *p = s->cur; s->cur += s->size; return p;
}
static inline void slab_free(Slab *s, void *p) { *(void **)p = s->free; s->free = p; }

/* [TYP-8] checked integer arithmetic (a language check, not a memory-model cost; used only where a
   model keeps run 3's checks so that its own diag/salvage twin differs from it by the count alone). */
static inline int64_t ck_add(int64_t a, int64_t b) {
#if defined(__clang__)
    int64_t r; if (UNLIKELY(__builtin_add_overflow(a, b, &r))) e_stop("integer overflow in '+'"); return r;
#else
    if ((b > 0 && a > INT64_MAX - b) || (b < 0 && a < INT64_MIN - b)) e_stop("integer overflow in '+'");
    return a + b;
#endif
}
static inline int64_t ck_mul(int64_t a, int64_t b) {
#if defined(__clang__)
    int64_t r; if (UNLIKELY(__builtin_mul_overflow(a, b, &r))) e_stop("integer overflow in '*'"); return r;
#else
    __int64 hi; __int64 lo = _mul128(a, b, &hi);
    if (hi != (lo >> 63)) e_stop("integer overflow in '*'");
    return lo;
#endif
}
/* [TYP-28] `%` is floor modulo; the caller checks a zero divisor where it is not proved non-zero. */
static inline int64_t fmod_i64(int64_t a, int64_t b) {
    int64_t r = a % b;
    if (r != 0 && ((r ^ b) < 0)) r += b;
    return r;
}
