/* ember_model.h: the pieces of Ember's runtime the C models use, written out so every model does the
   same work. Ember's Array is {ptr, len, cap} with 64-bit len and cap, first growth to 4 elements,
   then doubling (runtime/ember_rt/src/ember_rt.c, vec_reserve_at). An index is bounds-checked; a
   failed check panics (exit code 101 here). Integer overflow checks are left out of every model and
   every baseline, as in phase 3 (phase3-measurements.md section 4). */
#pragma once
#include "harness.h"
#include <string.h>

typedef struct { void **ptr; size_t len, cap; } EVec;   /* an Array of handles or pointers */

static void e_panic(const char *what) { fprintf(stderr, "panic: %s\n", what); exit(101); }
static inline void e_bounds(size_t i, size_t n) { if (i >= n) e_panic("index out of range"); }
static void evec_grow(EVec *v) {
    size_t cap = v->cap < 2 ? 4 : v->cap * 2;
    v->ptr = realloc(v->ptr, cap * sizeof(void *));
    v->cap = cap;
}
static inline void evec_push(EVec *v, void *x) {
    if (v->len == v->cap) evec_grow(v);
    v->ptr[v->len++] = x;
}
/* Array.remove(k): keeps the order, as Python's list.remove and Rust's Vec::remove do. */
static inline void evec_remove_at(EVec *v, size_t k) {
    e_bounds(k, v->len);
    memmove(&v->ptr[k], &v->ptr[k + 1], sizeof(void *) * (v->len - k - 1));
    v->len--;
}
static inline size_t evec_index_of(EVec *v, void *x) {
    for (size_t k = 0; k < v->len; k++) if (v->ptr[k] == x) return k;
    e_panic("value not in list");
    return 0;
}

/* A per-class slab: objects of one class are carved from 1 MiB chunks, in allocation order, with no
   per-block allocator header (what pymalloc's pools and D2/D3's stores do). Freed objects would go on
   a free list per class; nothing is freed in these workloads. Used when a model is built -DSLAB. */
typedef struct { char *cur, *end; } Slab;
static inline void *slab_alloc(Slab *s, size_t size) {
    if ((size_t)(s->end - s->cur) < size) {
        s->cur = malloc(1u << 20);
        s->end = s->cur + (1u << 20);
    }
    void *p = s->cur;
    s->cur += size;
    return p;
}
