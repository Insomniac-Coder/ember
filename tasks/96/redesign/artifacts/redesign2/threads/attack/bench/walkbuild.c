/* walkbuild.c: is the shared flag's marking walk on par with Rust's way of sharing a graph (A15)?
 *
 * Build 262,144 lean objects (8-byte header, 24 bytes of fields), each pointing at one random earlier object
 * (one counted link each), then share the whole graph with another thread once. Rust builds a graph it will
 * share out of `Arc`s from the start: every link is an atomic increment, and the share costs nothing. With
 * the flag, links are plain increments while one thread builds, and the share walks the graph once.
 * 32 rounds (build, share, free); the timer covers all of it.
 *
 *   -DMODEL=0  plain counts, no share (a program that never shares: the floor)
 *   -DMODEL=1  plain counts, then the marking walk from every object (the flag's price)
 *   -DMODEL=2  atomic increments for every link, no walk (Rust's `Arc::clone`, Swift)
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n walkbuild.c -o bin/walkbuild_n.exe */
#include "../../../../redesign/bench/harness.h"
#include <string.h>

#ifndef MODEL
#define MODEL 0
#endif
#define N (1u << 18)
#define ROUNDS 32
#define SHARED 0x80000000u

typedef struct Obj { uint32_t strong, type_index; int32_t hp, x; struct Obj *buddy; int64_t pad; } Obj;   /* 32 bytes */

static inline void link(Obj *from, Obj *to) {
    from->buddy = to;
#if MODEL == 2
    __atomic_fetch_add(&to->strong, 1u, __ATOMIC_RELAXED);
#else
    to->strong += 1;
#endif
}

static uint64_t mark_shared(Obj **roots, size_t n, Obj **stack) {
    uint64_t marked = 0;
    for (size_t r = 0; r < n; r++) {
        size_t top = 0;
        stack[top++] = roots[r];
        while (top) {
            Obj *o = stack[--top];
            if (o == NULL || (o->strong & SHARED)) continue;
            o->strong |= SHARED;
            marked++;
            stack[top++] = o->buddy;
        }
    }
    return marked;
}

int main(void) {
    h_start();
    h_seed(5);
    Obj **objs = (Obj **)malloc(N * sizeof(Obj *));
    Obj **stack = (Obj **)malloc(N * sizeof(Obj *));
    if (!objs || !stack) abort();
    uint64_t acc = 0;
    for (int round = 0; round < ROUNDS; round++) {
        for (uint32_t i = 0; i < N; i++) {
            Obj *o = (Obj *)malloc(sizeof(Obj));
            if (!o) abort();
            memset(o, 0, sizeof *o);
            o->strong = 1; o->type_index = 7; o->hp = 100; o->x = (int32_t)i;
            if (i) link(o, objs[h_rand() % i]);
            objs[i] = o;
        }
#if MODEL == 1
        acc += mark_shared(objs, N, stack);
#endif
        for (uint32_t i = 0; i < N; i++) { acc += (uint32_t)objs[i]->strong & 0xFF; free(objs[i]); }
    }
    static const char *names[] = {"walkbuild-0-plain-noshare", "walkbuild-1-plain-then-walk", "walkbuild-2-atomic-links"};
    h_report(names[MODEL], MODEL == 1 ? acc - (uint64_t)ROUNDS * N : acc);
    return 0;
}
