/* counts.c: what making class counts thread-safe costs a program (A15, the "shared flag").
 *
 * 262,144 objects (the ownership group's lean class: an 8-byte header, a 32-bit count and a 32-bit
 * type index, then 24 bytes of fields) are held by 1,048,576 watchers. Every frame every watcher reads
 * its target's `hp` and writes it; one watcher in sixteen re-targets to a random object (retain the new
 * one, release the old one). 32 frames. Same work, random numbers and checksum in every model.
 *
 *   -DMODEL=0  plain C: raw pointers, no header, no counts
 *   -DMODEL=1  lean class, plain counts (the ownership group's proposal, `ember_retain_plain`'s fast
 *              path: `strong - 1 < UINT32_MAX - 1`)
 *   -DMODEL=2  lean class with the thread-shared flag in the count's top bit (Koka's idea,
 *              phase2-rc-elision.md idea 2), folded into the same one compare: objects not shared
 *   -DMODEL=3  as 2, but every object was shared with another thread first (flag set): every count
 *              change takes the atomic path
 *   -DMODEL=4  every count atomic, always (what a language without the flag pays: Swift, Rust's Arc)
 *   -DSTEPRC=1 with any of 1..4: also a retain and a release of `t` on every step (32 million pairs),
 *              the count traffic of a handle held across a call, to make the per-operation cost visible
 *   -DMODEL=6  as 3 (every object shared), with the atomic path written inline after the one compare
 *              instead of in an out-of-line function (the saved version of model 3)
 *   -DMODEL=5  only the marking walk: build the objects (each points at another through `buddy`), then
 *              time one walk from a root that sets the flag on every object it reaches
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n counts.c -o counts_n.exe */
#include "../../../redesign/bench/harness.h"
#include <string.h>

#ifndef MODEL
#define MODEL 0
#endif
#define P_COUNT (1u << 18)
#define W_COUNT (1u << 20)
#define FRAMES 32

#if MODEL == 0
typedef struct Obj { int32_t hp; int32_t x; struct Obj *buddy; int64_t pad; } Obj;            /* 24 bytes */
#else
typedef struct { uint32_t strong, type_index; } Hdr;
typedef struct Obj { Hdr h; int32_t hp; int32_t x; struct Obj *buddy; int64_t pad; } Obj;     /* 32 bytes */
#endif

static __declspec(noinline) void fail(const char *what) { fprintf(stderr, "panic: %s\n", what); exit(101); }
static __declspec(noinline) void free_obj(Obj *o) { free(o); }

#define SHARED 0x80000000u

#if MODEL == 1
/* Out of line and returning, as the runtime's `ember_obj_retain` is (it also takes an empty handle). */
static __declspec(noinline) void retain_slow(Obj *o) { if (o->h.strong == UINT32_MAX) fail("count overflow"); }
static inline void retain(Obj *o) {
    if (o->h.strong - 1u < UINT32_MAX - 1u) { o->h.strong += 1; return; }
    retain_slow(o);
}
static inline void release(Obj *o) {
    if (o->h.strong > 1) { o->h.strong -= 1; return; }
    free_obj(o);
}
#elif MODEL == 6
static __declspec(noinline) void count_fail(Obj *o) { (void)o; fail("count overflow"); }
static inline void retain(Obj *o) {
    uint32_t c = o->h.strong;
    if (c - 1u < 0x7FFFFFFFu - 1u) { o->h.strong = c + 1; return; }
    if (c & SHARED) { __atomic_fetch_add(&o->h.strong, 1u, __ATOMIC_RELAXED); return; }
    count_fail(o);
}
static inline void release(Obj *o) {
    uint32_t c = o->h.strong;
    if ((int32_t)c > 1) { o->h.strong = c - 1; return; }
    if (c & SHARED) { if (__atomic_fetch_sub(&o->h.strong, 1u, __ATOMIC_ACQ_REL) == (SHARED | 1u)) free_obj(o); return; }
    free_obj(o);
}
#elif MODEL == 2 || MODEL == 3 || MODEL == 5
/* The flag is the top bit, so a shared object's count is out of the plain range [1, 2^31 - 2] and the
 * one compare the plain path already makes sends it to the slow path. */
static __declspec(noinline) void retain_slow(Obj *o) {
    if (o->h.strong & SHARED) { __atomic_fetch_add(&o->h.strong, 1u, __ATOMIC_RELAXED); return; }
    fail("count overflow");
}
static __declspec(noinline) void release_slow(Obj *o) {
    if (o->h.strong & SHARED) {
        if (__atomic_fetch_sub(&o->h.strong, 1u, __ATOMIC_ACQ_REL) == (SHARED | 1u)) free_obj(o);
        return;
    }
    free_obj(o);
}
static inline void retain(Obj *o) {
    if (o->h.strong - 1u < 0x7FFFFFFFu - 1u) { o->h.strong += 1; return; }
    retain_slow(o);
}
static inline void release(Obj *o) {
    if ((int32_t)o->h.strong > 1) { o->h.strong -= 1; return; }    /* signed: a flagged count is negative */
    release_slow(o);
}
#elif MODEL == 4
static inline void retain(Obj *o) { __atomic_fetch_add(&o->h.strong, 1u, __ATOMIC_RELAXED); }
static inline void release(Obj *o) {
    if (__atomic_fetch_sub(&o->h.strong, 1u, __ATOMIC_ACQ_REL) == 1u) free_obj(o);
}
#endif

static Obj *new_obj(uint32_t i) {
    Obj *o = (Obj *)malloc(sizeof(Obj));
    if (!o) abort();
    memset(o, 0, sizeof *o);
#if MODEL != 0
    o->h.strong = 1; o->h.type_index = 7;
#endif
    o->hp = 1000; o->x = (int32_t)(i & 1023);
    return o;
}

#if MODEL == 3 || MODEL == 5 || MODEL == 6
/* The marking walk a share runs once: every object reachable from the shared one gets the flag.
 * An object already flagged stops the walk. The stack is the walk's own. */
static uint64_t mark_shared(Obj **roots, size_t n) {
    Obj **stack = (Obj **)malloc(P_COUNT * sizeof(Obj *));
    if (!stack) abort();
    uint64_t marked = 0;
    for (size_t r = 0; r < n; r++) {
        size_t top = 0;
        stack[top++] = roots[r];
        while (top) {
            Obj *o = stack[--top];
            if (o == NULL || (o->h.strong & SHARED)) continue;
            o->h.strong |= SHARED;
            marked++;
            stack[top++] = o->buddy;
        }
    }
    free(stack);
    return marked;
}
#endif

int main(void) {
#if MODEL != 5
    h_start();
#endif
    h_seed(42);
    Obj **objs = (Obj **)malloc(P_COUNT * sizeof(Obj *));
    Obj **ws = (Obj **)malloc(W_COUNT * sizeof(Obj *));
    if (!objs || !ws) abort();
    for (uint32_t i = 0; i < P_COUNT; i++) objs[i] = new_obj(i);
    for (uint32_t i = 0; i < P_COUNT; i++) {
        objs[i]->buddy = objs[(i * 2654435761u + 1) & (P_COUNT - 1)];   /* a graph: each points at one */
#if MODEL != 0
        retain(objs[i]->buddy);
#endif
    }
    for (uint32_t i = 0; i < W_COUNT; i++) {
        Obj *t = objs[h_rand() & (P_COUNT - 1)];
#if MODEL != 0
        retain(t);
#endif
        ws[i] = t;
    }
#if MODEL == 5
    h_start();                                   /* time only the walk */
    uint64_t marked = mark_shared(objs, P_COUNT);
    h_report("counts-5-markwalk", marked);
    return 0;
#else
#if MODEL == 3 || MODEL == 6
    if (mark_shared(objs, P_COUNT) != P_COUNT) fail("walk");
#endif
    uint64_t acc = 0;
    for (uint32_t f = 0; f < FRAMES; f++) {
        for (uint32_t i = 0; i < W_COUNT; i++) {
            Obj *t = ws[i];
#if defined(STEPRC) && MODEL != 0
            retain(t);
#endif
            t->hp -= 1;
            acc += (uint64_t)(uint32_t)(t->hp ^ t->x);
            if ((h_rand() & 15) == 0) {                     /* ws[i] = objs[k]: a handle store */
                Obj *n = objs[h_rand() & (P_COUNT - 1)];
#if MODEL != 0
                retain(n);
                release(t);
#endif
                ws[i] = n;
            }
#if defined(STEPRC) && MODEL != 0
            release(t);
#endif
        }
    }
    static const char *names[] = {"counts-0-plainC", "counts-1-lean-plain", "counts-2-lean-flag",
                                  "counts-3-lean-flag-shared", "counts-4-atomic-always", "", "counts-6-flag-shared-inline"};
#ifdef STEPRC
    static const char *stepnames[] = {"", "counts-1s-lean-plain", "counts-2s-lean-flag",
                                      "counts-3s-lean-flag-shared", "counts-4s-atomic-always", "", "counts-6s-flag-shared-inline"};
    h_report(stepnames[MODEL], acc);
#else
    h_report(names[MODEL], acc);
#endif
    return 0;
#endif
}
