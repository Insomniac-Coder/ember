/* The C side of accept_static_inline_functions_through_the_shim.em: a header-only library. */
#ifndef INLINE_MATH_H
#define INLINE_MATH_H

#ifndef INLINE_MATH_SCALE
#define INLINE_MATH_SCALE 1
#endif
#define INLINE_MATH_FACTOR (INLINE_MATH_SCALE * 10)

typedef struct Pair { int first; int second; } Pair;

static inline int inline_scale(int x) { return x * INLINE_MATH_SCALE; }
static inline Pair inline_swap(Pair p) { Pair q = { p.second, p.first }; return q; }
static inline void inline_store(int* out, int value) { *out = value; }
static int inline_twice(int (*f)(int), int x) { return f(f(x)); }
#endif
