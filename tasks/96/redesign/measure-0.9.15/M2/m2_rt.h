/* m2_rt.h: the runtime pieces M2's C models share, and the one class both workloads use (`Node`).
   Spec: docs/spec-source/Ember_v0.9.15_Hardened_1.md; rules: tasks/96/redesign/run4/v5/rules.md (R1..R18).
   Portable to clang -O2 and MSVC cl /O2: no statement expressions, no typeof, no VLAs.

   Every check the language keeps goes through KEEP or a checked-arithmetic function; -DPLAIN removes
   all of them and every weak count and header test (plain C in the model's layout, "plain_same").

   Switches read here (each model file says which of them it is built with):
     -DPLAIN             plain C in the model's layout: no checks, no counts, no header tests
     -DPARENT_UNCOUNTED  salvage: a parent link proved by [WK-17] condition 1 holds no weak count
     -DDROPCLASS         `class Node` has a `drop` method of its own: [WK-17] condition 1 fails, so every
                         parent-link read tests the target's header (R11), and no deferred-drop region */
#pragma once
#include "harness.h"
#include <string.h>
#if defined(_MSC_VER) && !defined(__clang__)
#include <intrin.h>
#endif

#if defined(_MSC_VER)
#define NOINLINE __declspec(noinline)
#define NORETURN __declspec(noreturn)
#define THREAD_LOCAL __declspec(thread)
#else
#define NOINLINE __attribute__((noinline))
#define NORETURN __attribute__((noreturn))
#define THREAD_LOCAL _Thread_local
#endif
#if defined(__clang__) || defined(__GNUC__)
#define UNLIKELY(c) __builtin_expect(!!(c), 0)
#else
#define UNLIKELY(c) (c)
#endif
#define SIZE_CHECK(name, cond) typedef char name[(cond) ? 1 : -1]

#if defined(PARENT_UNCOUNTED) && defined(DROPCLASS)
#error "PARENT_UNCOUNTED needs [WK-17] condition 1, which a class with its own drop method fails"
#endif

static NORETURN NOINLINE void e_panic(const char *m) {
    fflush(stdout);
    fprintf(stderr, "panic: %s\n", m);
    exit(101);
}

#ifdef PLAIN
#define KEEP(c, m) ((void)0)
static inline int64_t add_i64(int64_t a, int64_t b) { return a + b; }
static inline int64_t mul_i64(int64_t a, int64_t b) { return a * b; }
static inline int64_t fmod_i64(int64_t a, int64_t b) { return a % b; }
#else
/* a check that stops the program with Ember's message when `c` is false */
#define KEEP(c, m) do { if (UNLIKELY(!(c))) e_panic(m); } while (0)
/* [TYP-8]: integer overflow stops the program in every profile */
static inline int64_t add_i64(int64_t a, int64_t b) {
#if defined(__clang__) || defined(__GNUC__)
    int64_t r;
    if (UNLIKELY(__builtin_add_overflow(a, b, &r))) e_panic("integer overflow in '+'");
    return r;
#else
    if ((b > 0 && a > INT64_MAX - b) || (b < 0 && a < INT64_MIN - b)) e_panic("integer overflow in '+'");
    return a + b;
#endif
}
static inline int64_t mul_i64(int64_t a, int64_t b) {
#if defined(__clang__) || defined(__GNUC__)
    int64_t r;
    if (UNLIKELY(__builtin_mul_overflow(a, b, &r))) e_panic("integer overflow in '*'");
    return r;
#else
    int64_t hi, lo = _mul128(a, b, &hi);
    if (hi != (lo >> 63)) e_panic("integer overflow in '*'");
    return lo;
#endif
}
/* [TYP-28]: `%` on signed integers is floor modulo; the divisor here is proved non-zero by the caller */
static inline int64_t fmod_i64(int64_t a, int64_t b) {
    int64_t r = a % b;
    if (r != 0 && ((r ^ b) < 0)) r += b;
    return r;
}
#endif
#define BOUNDS(i, len) KEEP((uint64_t)(i) < (uint64_t)(len), "index out of bounds")

/* Ember's Array header {ptr, int64 len, int64 cap}; the first growth is to 4, then doubling. */
static void arr_grow(void **ptr, int64_t *cap, size_t esz) {
    int64_t nc = *cap < 4 ? 4 : *cap * 2;
    void *p;
    KEEP(nc <= (int64_t)(PTRDIFF_MAX / (ptrdiff_t)esz), "capacity overflow");
    p = realloc(*ptr, (size_t)nc * esz);
    if (p == NULL) e_panic("out of memory");                 /* kept in PLAIN too: C checks malloc */
    *ptr = p; *cap = nc;
}
#define ARR(T) struct { T *ptr; int64_t len, cap; }
#define PUSH(a, v) do { if ((a)->len == (a)->cap) arr_grow((void **)&(a)->ptr, &(a)->cap, sizeof *(a)->ptr); \
                        (a)->ptr[(a)->len++] = (v); } while (0)

/* The slab for one object size (the runtime allocator of [OBJ-4], run 3's): chunks of 4,096 blocks
   carved in order, then a free list. */
typedef struct { char *cur, *end; void *free; size_t size; } Slab;
static inline void *slab_alloc(Slab *s) {
    void *p;
    if (s->free) { p = s->free; s->free = *(void **)p; return p; }
    if (s->cur == s->end) {
        s->cur = (char *)malloc(s->size * 4096);
        if (s->cur == NULL) e_panic("out of memory");
        s->end = s->cur + s->size * 4096;
    }
    p = s->cur; s->cur += s->size; return p;
}
static inline void slab_free(Slab *s, void *p) { *(void **)p = s->free; s->free = p; }

/* ---------------------------------------------------------------------------------------------
   class Node:
       value: i32
       parent: weakref Node = None        # a parent link (R11): set by the program, `weakref this`
       children: Array[Node] = []         # each child has this one owner

   [OBJ-1]: a `weakref` link may name a Node, so its header is 12 bytes: the count, the type index and
   the weak count. Node = {count, type, weak, value, parent, children{ptr,len,cap}} = 48 bytes, the
   size run 3's W2 model had (its third header word was the same weak count).
   [OBJ-3]: the weak count starts at 1 on behalf of all owners; [WK-2]/[WK-3]: the object is ended when
   its count reaches 0, before its drop runs; the block is freed when the weak count reaches 0.
   A one-owner object is not counted ([RC-1]): its count stays 1 while it lives.
   --------------------------------------------------------------------------------------------- */
typedef struct Node Node;
typedef ARR(Node *) NodeArr;
struct Node { uint32_t count, type, weak; int32_t value; Node *parent; NodeArr children; };
SIZE_CHECK(node_is_48_bytes, sizeof(Node) == 48);

static Slab node_slab = { NULL, NULL, NULL, sizeof(Node) };

static Node *node_new(int32_t value) {               /* Node(value=...) */
    Node *n = (Node *)slab_alloc(&node_slab);
    n->count = 1; n->type = 1; n->weak = 1;          /* [OBJ-3]: weak 1 on behalf of the owners */
    n->value = value; n->parent = NULL;
    n->children.ptr = NULL; n->children.len = 0; n->children.cap = 0;
    return n;
}

/* The block's free when its weak count reaches 0 ([OBJ-3]); a deferred-free region (salvage) hooks it. */
#ifndef BLOCK_FREE
#define BLOCK_FREE(n) slab_free(&node_slab, (n))
#endif

/* Making a `weakref` link: +1 on the target's weak count, compared with its maximum first
   ([WK-11], [RC-4], R11; the stop's text from rules.md section 2). */
static inline void weak_add(Node *t) {
#ifndef PLAIN
    KEEP(t->weak != 0xFFFFFFFFu, "too many `weakref` links to a `Node` (4,294,967,295)");
    t->weak++;
#else
    (void)t;
#endif
}
/* Ending a `weakref` link, or giving it another target: -1; the block goes at 0 ([WK-11], [OBJ-3]).
   This is Rust's `Weak` drop: a decrement and a test for zero. */
static inline void weak_sub(Node *t) {
#ifndef PLAIN
    if (--t->weak == 0) BLOCK_FREE(t);
#else
    (void)t;
#endif
}
/* Opening a link, `if w exists as x:` ([WK-12], R11): the test for no link, then one test of the
   target's header (does it still live?). */
static inline Node *open_link(Node *w) {
#ifdef PLAIN
    return w;
#else
    return (w != NULL && w->count != 0) ? w : NULL;
#endif
}

/* A parent-link read, `if n.parent exists as p:`. Proved by [WK-17] (both conditions hold in both
   workloads unless DROPCLASS): a plain load and a test for no link, no header test. */
#ifdef DROPCLASS
#define PARENT_READ(n) open_link((n)->parent)
#else
#define PARENT_READ(n) ((n)->parent)
#endif

/* A parent link's weak count (R11: "Parent links pay it too"). */
#if defined(PLAIN) || defined(PARENT_UNCOUNTED)
#define PARENT_LINK_ADD(t) ((void)0)
#define PARENT_LINK_SUB(t) ((void)0)
#else
#define PARENT_LINK_ADD(t) weak_add(t)
#define PARENT_LINK_SUB(t) weak_sub(t)
#endif

/* `holder.children.index_of(x).unwrap()` */
static inline int64_t index_of_unwrap(NodeArr *a, Node *x) {
    int64_t k;
    for (k = 0; k < a->len; k++)
        if (a->ptr[k] == x) return k;
    KEEP(0, "called `unwrap` on `None`");
    return 0;
}
/* `holder.children.swap_remove(k)`: the element is handed out to the caller */
static inline Node *swap_remove(NodeArr *a, int64_t k) {
    Node *c;
    BOUNDS(k, a->len);
    c = a->ptr[k];
    a->ptr[k] = a->ptr[--a->len];
    return c;
}

#ifdef DROPCLASS
static int64_t g_drops;                      /* `fn drop(this): DROPS += 1`, the class's own drop */
#define DROP_METHOD(n) (g_drops++)
#else
#define DROP_METHOD(n) ((void)0)
#endif

static void node_end(Node *n);               /* the last owner lets `n` go; each model defines it */

/* The end of a Node: ended first ([WK-3]), then its drop method, then its fields in order:
   `parent` (the link's -1), `children` (each child ends); then the owners' share of the weak count,
   which frees the block unless a link still holds it ([OBJ-3]). */
static void node_drop(Node *n) {
    int64_t k;
    n->count = 0;
    DROP_METHOD(n);
    if (n->parent) PARENT_LINK_SUB(n->parent);
    for (k = 0; k < n->children.len; k++) node_end(n->children.ptr[k]);
    free(n->children.ptr);
#ifdef PLAIN
    slab_free(&node_slab, n);
#else
    weak_sub(n);
#endif
}
