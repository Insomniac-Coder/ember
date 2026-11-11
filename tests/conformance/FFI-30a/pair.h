/* The C side of the FFI-30a tests. */
#ifndef PAIR_H
#define PAIR_H

typedef struct Pair { int left; int right; } Pair;

static inline int pair_sum(Pair p) { return p.left + p.right; }
#endif
