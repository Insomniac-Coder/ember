/* m4.h: the pieces every C program of job M4 shares. Portable to clang -O2 and MSVC cl /O2: no
   statement expressions, no __typeof__, no VLAs; GCC builtins only behind __clang__.
   A plain C program uses only the macros (NOINLINE, LIKELY, RESTRICT, ...) and the Array type; the
   checks (ck_add, ck_sub, e_panic, the bounds tests) are what the model programs add. */
#pragma once
#include "harness.h"
#include <string.h>

#if defined(__clang__) || defined(__GNUC__)
#define NOINLINE __attribute__((noinline))
#define LIKELY(c) __builtin_expect(!!(c), 1)
#define UNLIKELY(c) __builtin_expect(!!(c), 0)
#else
#define NOINLINE __declspec(noinline)
#define LIKELY(c) (c)
#define UNLIKELY(c) (c)
#endif
#define NORETURN __declspec(noreturn)
#define FORCEINLINE __forceinline
#define RESTRICT __restrict          /* C's `restrict`: what Ember's noalias is on the C backend ([BRW-22]) */
#define TLS __declspec(thread)

/* [PAN-1] a panic prints and ends the process, with no unwinding (exit(101) as run 3's models, so no
   abort() dialog can reach the owner's screen; no check in any workload fires). */
static NORETURN NOINLINE void e_panic(const char *m) {
    fflush(stdout);
    fprintf(stderr, "panic: %s\n", m);
    exit(101);
}

/* [TYP-8] integer overflow panics in every profile. */
static FORCEINLINE int64_t ck_add(int64_t a, int64_t b) {
    int64_t r;
#if defined(__clang__) || defined(__GNUC__)
    if (UNLIKELY(__builtin_add_overflow(a, b, &r))) e_panic("integer overflow in '+'");
#else
    r = (int64_t)((uint64_t)a + (uint64_t)b);
    if (((a ^ r) & (b ^ r)) < 0) e_panic("integer overflow in '+'");
#endif
    return r;
}
static FORCEINLINE int64_t ck_sub(int64_t a, int64_t b) {
    int64_t r;
#if defined(__clang__) || defined(__GNUC__)
    if (UNLIKELY(__builtin_sub_overflow(a, b, &r))) e_panic("integer overflow in '-'");
#else
    r = (int64_t)((uint64_t)a - (uint64_t)b);
    if (((a ^ b) & (a ^ r)) < 0) e_panic("integer overflow in '-'");
#endif
    return r;
}

/* Ember's Array header {ptr, int64 len, int64 cap}; growth 4, 8, 16, ... (run 3's ember_rt.h). */
typedef struct { int64_t *ptr; int64_t len, cap; } ArrInt;
typedef struct { double *ptr; int64_t len, cap; } ArrF64;
static NOINLINE void arr_grow(void **ptr, int64_t *cap, size_t esz) {
    int64_t nc = *cap < 4 ? 4 : *cap * 2;
    void *p = realloc(*ptr, (size_t)nc * esz);
    if (!p) e_panic("out of memory");
    *ptr = p; *cap = nc;
}
static FORCEINLINE void push_int(ArrInt *a, int64_t v) {
    if (a->len == a->cap) arr_grow((void **)&a->ptr, &a->cap, sizeof *a->ptr);
    a->ptr[a->len++] = v;
}

/* The object a method is called on comes from elsewhere in a real program; a pointer read back from a
   volatile has a provenance the C compiler cannot see (a fresh `calloc` result would tell it that
   nothing else points there, which no caller of a method can assume). */
static void *volatile g_launder;
static FORCEINLINE void *launder(void *p) { g_launder = p; return g_launder; }
