/* m3_rt.c: the out-of-line half of m3_rt.h, compiled as its own translation unit (as Ember's
   runtime is), so no program can inline these calls. */
#include "m3_rt.h"
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>

void m3_panic(const char *msg) {
    fprintf(stderr, "panic: %s\n", msg);
    fflush(stderr);
    exit(101);
}

/* ---------------------------------------------------------------- RtArena
   Ported from runtime/ember_rt/src/ember_rt.c (ember_arena_new, arena_chunk_take, arena_alloc_raw,
   ember_arena_alloc_uninit, ember_arena_reset, ember_arena_free). The runtime takes its chunks from
   ember_alloc (the global allocator); malloc here, which gives 16-byte alignment on Windows x64. */
#define RT_GROWTH ((size_t)1024 * (size_t)1024)
#define RT_MAX_ALIGN ((size_t)16)

static int rt_power_of_two(size_t v) { return v != 0 && (v & (v - 1)) == 0; }

static RtChunk *rt_chunk_new(size_t capacity, size_t align) {
    if (align > RT_MAX_ALIGN) m3_panic("arena alignment above 16 is not modelled");
    RtChunk *c = (RtChunk *)malloc(sizeof(RtChunk));
    if (c == NULL) m3_panic("out of memory");
    c->next = NULL;
    c->capacity = capacity;
    c->used = 0;
    c->align = align;
    c->data = (unsigned char *)malloc(capacity);
    if (c->data == NULL) m3_panic("out of memory");
    return c;
}

RtArena *rt_arena_new(size_t initial_capacity) {
    RtArena *a = (RtArena *)malloc(sizeof(RtArena));
    if (a == NULL) m3_panic("out of memory");
    a->first = NULL;
    a->current = NULL;
    if (initial_capacity != 0) {
        a->first = rt_chunk_new(initial_capacity, RT_MAX_ALIGN);
        a->current = a->first;
    }
    return a;
}

static void *rt_chunk_take(RtChunk *c, size_t size, size_t align) {
    if (c->align < align || c->used > SIZE_MAX - (align - 1)) return NULL;
    size_t offset = (c->used + align - 1) & ~(align - 1);
    if (offset > c->capacity || size > c->capacity - offset) return NULL;
    c->used = offset + size;
    return c->data + offset;
}

static void *rt_arena_alloc_raw(RtArena *a, size_t size, size_t align) {
    if (a == NULL || !rt_power_of_two(align)) m3_panic("invalid arena allocation");
    if (size == 0) size = 1;
    RtChunk *c = a->current ? a->current : a->first;
    while (c != NULL) {
        void *slot = rt_chunk_take(c, size, align);
        if (slot != NULL) {
            a->current = c;
            return slot;
        }
        c = c->next;
    }
    size_t capacity = size > RT_GROWTH ? size : RT_GROWTH;
    size_t chunk_align = align > RT_MAX_ALIGN ? align : RT_MAX_ALIGN;
    RtChunk *fresh = rt_chunk_new(capacity, chunk_align);
    if (a->first == NULL) {
        a->first = fresh;
    } else {
        RtChunk *tail = a->first;
        while (tail->next != NULL) tail = tail->next;
        tail->next = fresh;
    }
    a->current = fresh;
    void *slot = rt_chunk_take(fresh, size, align);
    if (slot == NULL) m3_panic("arena allocation overflow");
    return slot;
}

void *rt_arena_alloc_uninit(RtArena *a, size_t count, size_t elem_size, size_t align) {
    if (elem_size != 0 && count > SIZE_MAX / elem_size) m3_panic("arena allocation overflow");
    return rt_arena_alloc_raw(a, count * elem_size, align);
}

void *rt_arena_reallocate(RtArena *a, void *p, size_t old_size, size_t new_size, size_t align) {
    if (new_size < old_size) m3_panic("arena reallocate to a smaller size is not modelled");
    RtChunk *c = a->current;
    if (p != NULL && c != NULL && (unsigned char *)p + old_size == c->data + c->used &&
        new_size - old_size <= c->capacity - c->used) {
        c->used += new_size - old_size;
        return p;
    }
    void *q = rt_arena_alloc_raw(a, new_size, align);
    if (old_size != 0) memcpy(q, p, old_size);
    return q;
}

void rt_arena_deallocate(RtArena *a, void *p, size_t size, size_t align) {
    (void)a; (void)p; (void)size; (void)align;
}

void rt_arena_reset(RtArena *a) {
    if (a == NULL) return;
    for (RtChunk *c = a->first; c != NULL; c = c->next) c->used = 0;
    a->current = a->first;
}

void rt_arena_free(RtArena *a) {
    if (a == NULL) return;
    RtChunk *c = a->first;
    while (c != NULL) {
        RtChunk *next = c->next;
        free(c->data);
        free(c);
        c = next;
    }
    free(a);
}

/* ---------------------------------------------------------------- HArena */
static HChunk *h_chunk_new(HArena *a, size_t bytes) {
    HChunk *c = (HChunk *)VirtualAlloc(NULL, bytes, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
    if (c == NULL) m3_panic("out of memory");
    if (((uintptr_t)c & (H_CHUNK - 1)) != 0) m3_panic("VirtualAlloc gave a region off a 64 KB boundary");
    c->arena = a;
    c->next = NULL;
    c->cur = (unsigned char *)c + sizeof(HChunk);
    c->end = (unsigned char *)c + bytes;
    c->bytes = bytes;
    return c;
}

HArena *h_arena_new(void) {
    HArena *a = (HArena *)malloc(sizeof(HArena));
    if (a == NULL) m3_panic("out of memory");
    a->big = a->bigfree = NULL;
    a->first = a->current = h_chunk_new(a, H_CHUNK);
    return a;
}

void *h_arena_slow(HArena *a, size_t size, size_t align) {
    if (!rt_power_of_two(align) || align > 64) m3_panic("invalid arena allocation");
    if (size > H_CHUNK - sizeof(HChunk)) {
        /* A block too big for a chunk takes a region of its own, starting 64 bytes in (so the mask
           finds the header): one a reset kept when it is big enough, else a new one. */
        size_t bytes = (size + sizeof(HChunk) + H_CHUNK - 1) & ~(H_CHUNK - 1);
        if (bytes < size) m3_panic("arena allocation overflow");
        HChunk *b = NULL, **link = &a->bigfree;
        for (; *link != NULL; link = &(*link)->next) {
            if ((*link)->bytes >= bytes) {
                b = *link;
                *link = b->next;
                break;
            }
        }
        if (b == NULL) b = h_chunk_new(a, bytes);
        void *p = (unsigned char *)b + sizeof(HChunk);
        b->cur = b->end;
        b->next = a->big;
        a->big = b;
        return p;
    }
    if (a->current->next != NULL) { /* a chunk kept by the last reset, empty: any such block fits */
        a->current = a->current->next;
        return h_arena_bump(a, size, align);
    }
    HChunk *fresh = h_chunk_new(a, H_CHUNK);
    a->current->next = fresh; /* the current chunk is the list's last here */
    a->current = fresh;
    return h_arena_bump(a, size, align);
}

void h_arena_reset(HArena *a) {
    for (HChunk *c = a->first; c != NULL; c = c->next) c->cur = (unsigned char *)c + sizeof(HChunk);
    a->current = a->first;
    while (a->big != NULL) { /* kept for the next big block, as the runtime arena keeps its chunks */
        HChunk *n = a->big->next;
        a->big->next = a->bigfree;
        a->bigfree = a->big;
        a->big = n;
    }
}

void h_arena_free(HArena *a) {
    h_arena_reset(a);
    while (a->bigfree != NULL) {
        HChunk *n = a->bigfree->next;
        VirtualFree(a->bigfree, 0, MEM_RELEASE);
        a->bigfree = n;
    }
    HChunk *c = a->first;
    while (c != NULL) {
        HChunk *n = c->next;
        VirtualFree(c, 0, MEM_RELEASE);
        c = n;
    }
    free(a);
}

/* ---------------------------------------------------------------- Pool */
void pool_init(Pool *p, size_t bsize) {
    if (bsize < sizeof(void *) || bsize % sizeof(void *) != 0) m3_panic("bad pool block size");
    p->free = NULL;
    p->cur = p->end = NULL;
    p->bsize = bsize;
    p->slabs = NULL;
}

void *pool_slow(Pool *p) {
    PoolSlab *s = (PoolSlab *)VirtualAlloc(NULL, POOL_SLAB, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
    if (s == NULL) return NULL;
    if (((uintptr_t)s & (POOL_SLAB - 1)) != 0) m3_panic("VirtualAlloc gave a region off a 64 KB boundary");
    s->pool = p;
    s->next = p->slabs;
    p->slabs = s;
    p->cur = (unsigned char *)s + POOL_HEADER;
    p->end = (unsigned char *)s + POOL_SLAB;
    void *b = p->cur;
    p->cur += p->bsize;
    return b;
}

void pool_destroy(Pool *p) {
    while (p->slabs != NULL) {
        PoolSlab *n = p->slabs->next;
        VirtualFree(p->slabs, 0, MEM_RELEASE);
        p->slabs = n;
    }
    p->free = NULL;
    p->cur = p->end = NULL;
}
