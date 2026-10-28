/* m3_rt.h: the memory managers M3's C programs share (spec 0.9.15_Hardened_1, R3, verdict 47).
   One copy of each, so every program that uses a manager runs the same code:

   - RtArena: a port of today's runtime arena (runtime/ember_rt/src/ember_rt.c, section "arenas":
     chunks of 1 MB from the global allocator, a list of chunks, a reset that rewinds them). Its entry
     points live in m3_rt.c, a separate translation unit, as the runtime's do, so the compiler cannot
     inline them. rt_arena_bump / rt_arena_grow_inl are the same arena's fast path written inline,
     for the forms that model an `allocate` compiled into the program's own code.
   - HArena: verdict 47's way (b) for an arena: chunks on 64 KB boundaries (VirtualAlloc's allocation
     granularity), whose first 64 bytes hold the arena's address, so a block's address finds its
     arena: `p & ~0xFFFF`.
   - Pool: a memory manager that does free single values (one block size, a free list, slabs of
     64 KB). Every slab starts with a 64-byte header holding its pool and the next slab (the list a
     C programmer keeps to free the slabs at the end); only way (b) reads the pool from it.

   Portable to MSVC: no GCC-only extension outside the macros below, no VLA. */
#pragma once
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#if defined(_MSC_VER)
#define M3_NOINLINE __declspec(noinline)
#else
#define M3_NOINLINE __attribute__((noinline))
#endif
#if defined(__clang__) || defined(__GNUC__)
#define M3_LIKELY(x) __builtin_expect(!!(x), 1)
#else
#define M3_LIKELY(x) (x)
#endif

/* An Ember panic: "panic: <msg>" on stderr, exit code 101. */
void m3_panic(const char *msg);

/* ---------------------------------------------------------------- RtArena (today's runtime) */
typedef struct RtChunk {
    struct RtChunk *next;
    unsigned char *data;
    size_t capacity, used, align;
} RtChunk;
typedef struct RtArena { RtChunk *first, *current; } RtArena;

RtArena *rt_arena_new(size_t initial_capacity);                                   /* ember_arena_new */
void *rt_arena_alloc_uninit(RtArena *a, size_t count, size_t elem_size, size_t align); /* ember_arena_alloc_uninit */
/* `Arena`'s `Allocator.reallocate` ([ALC-2]) as the runtime would give it: the block grown in place
   when it is the arena's last, else a new block holding the old bytes (the old one stays, [ARN-2]). */
void *rt_arena_reallocate(RtArena *a, void *p, size_t old_size, size_t new_size, size_t align);
/* `Arena`'s `Allocator.deallocate` ([ALC-2]): "gives its block back to the arena, which frees
   nothing one by one" ([ARN-2]). An empty body, in another translation unit. */
void rt_arena_deallocate(RtArena *a, void *p, size_t size, size_t align);
void rt_arena_reset(RtArena *a);                                                  /* ember_arena_reset */
void rt_arena_free(RtArena *a);                                                   /* ember_arena_free */

/* The runtime's chunk_take, inline: the current chunk has room, else the out-of-line entry. Every
   chunk is aligned to at least 16, so an alignment up to 16 needs no chunk test. */
static inline void *rt_arena_bump(RtArena *a, size_t size, size_t align) {
    RtChunk *c = a->current;
    if (M3_LIKELY(c != NULL)) {
        size_t off = (c->used + align - 1) & ~(align - 1);
        if (M3_LIKELY(off <= c->capacity && size <= c->capacity - off && (align <= 16 || c->align >= align))) {
            c->used = off + size;
            return c->data + off;
        }
    }
    return rt_arena_alloc_uninit(a, 1, size, align);
}

/* Growth, inline (new_size >= old_size): in place when `p` is the current chunk's last block. */
static inline void *rt_arena_grow_inl(RtArena *a, void *p, size_t old_size, size_t new_size, size_t align) {
    RtChunk *c = a->current;
    if (p != NULL && c != NULL && (unsigned char *)p + old_size == c->data + c->used &&
        new_size - old_size <= c->capacity - c->used) {
        c->used += new_size - old_size;
        return p;
    }
    void *q = rt_arena_bump(a, new_size, align);
    if (old_size != 0) memcpy(q, p, old_size);
    return q;
}

/* ---------------------------------------------------------------- HArena (verdict 47, way (b)) */
#define H_CHUNK ((size_t)1 << 16) /* VirtualAlloc gives every region on a 64 KB boundary */
typedef struct HArena HArena;
typedef struct HChunk {           /* 64 bytes, at the chunk's first byte */
    HArena *arena;                /* the instance every block of this chunk came from */
    struct HChunk *next;
    unsigned char *cur, *end;     /* a normal chunk's end is its start + 64 KB */
    size_t bytes;                 /* the region's size, for VirtualFree */
    size_t pad[3];
} HChunk;
struct HArena { HChunk *first, *current, *big, *bigfree; }; /* big: regions of one big block each, in use; bigfree: kept by a reset for the next big block */

HArena *h_arena_new(void);
void *h_arena_slow(HArena *a, size_t size, size_t align);
void h_arena_reset(HArena *a);
void h_arena_free(HArena *a);

/* The arena a block came from: one AND and one load. Valid for every block an HArena gives: a
   normal chunk's blocks lie in its first 64 KB, and a block too big for one takes a chunk of its own
   and starts 64 bytes after that chunk's first byte. */
static inline HArena *h_arena_of(const void *p) {
    return ((const HChunk *)((uintptr_t)p & ~(uintptr_t)(H_CHUNK - 1)))->arena;
}
static inline void *h_arena_bump(HArena *a, size_t size, size_t align) {
    HChunk *c = a->current;
    uintptr_t q = ((uintptr_t)c->cur + align - 1) & ~(uintptr_t)(align - 1);
    if (M3_LIKELY(q <= (uintptr_t)c->end && size <= (uintptr_t)c->end - q)) {
        c->cur = (unsigned char *)q + size;
        return (void *)q;
    }
    return h_arena_slow(a, size, align);
}
/* The block of an empty list: a real address inside a chunk (its first data byte), never read,
   so that the list's own pointer finds its arena before its first block exists. */
static inline void *h_arena_empty(HArena *a) {
    return (unsigned char *)a->current + sizeof(HChunk);
}
static inline void *h_arena_grow_inl(HArena *a, void *p, size_t old_size, size_t new_size, size_t align) {
    HChunk *c = a->current;
    if ((unsigned char *)p + old_size == c->cur && new_size - old_size <= (size_t)(c->end - c->cur)) {
        c->cur += new_size - old_size;
        return p;
    }
    void *q = h_arena_bump(a, new_size, align);
    if (old_size != 0) memcpy(q, p, old_size);
    return q;
}

/* ---------------------------------------------------------------- Pool (frees single values) */
#define POOL_SLAB ((size_t)1 << 16)
#define POOL_HEADER 64
typedef struct Pool Pool;
typedef struct PoolSlab { Pool *pool; struct PoolSlab *next; } PoolSlab; /* in the slab's first 64 bytes */
struct Pool {
    void *free;               /* the free list, linked through each free block's first word */
    unsigned char *cur, *end; /* the current slab's unused tail */
    size_t bsize;
    PoolSlab *slabs;
};

void pool_init(Pool *p, size_t bsize);
void *pool_slow(Pool *p); /* a new slab; NULL when the system has no memory */
void pool_destroy(Pool *p);

static inline void *pool_alloc(Pool *p) {
    void *b = p->free;
    if (b != NULL) {
        p->free = *(void **)b;
        return b;
    }
    if (M3_LIKELY((size_t)(p->end - p->cur) >= p->bsize)) {
        b = p->cur;
        p->cur += p->bsize;
        return b;
    }
    return pool_slow(p);
}
static inline void pool_free(Pool *p, void *b) {
    *(void **)b = p->free;
    p->free = b;
}
/* Way (b) for the pool: the slab's header holds the pool. */
static inline Pool *pool_of(const void *b) {
    return ((const PoolSlab *)((uintptr_t)b & ~(uintptr_t)(POOL_SLAB - 1)))->pool;
}
