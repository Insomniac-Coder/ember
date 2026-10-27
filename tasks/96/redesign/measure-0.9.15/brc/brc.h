/* brc.h (version 2): ways to count the owners of a `thread_shared` object behind one interface, so one
   stress test and one set of timed patterns run on each (the lead's, 2026-10-27; version 1 is brc_v1.h).

   COUNT_ATOMIC        today's rule (0.9.15: [RC-4], [RC-5] point 2, [WK-12], [RT-8], [RT-10]): one owner
                       count, every +1 and -1 a locked instruction; a `weakref` opening is a compare-exchange
                       loop that raises the count only while it is above 0; the weak count is locked too.
   COUNT_BIASED        biased reference counting (Choi, Shull and Torrellas, PACT 2018), with the id of the
                       thread that made the object kept in the object, next to its counts (version 1).
   COUNT_BIASED_CHUNK  the same counting; the maker's id is read from the header of the 64 KiB chunk the
                       object's block came from, which is written once when the chunk is made (fix 1).
   COUNT_BIASED_PUB    COUNT_BIASED_CHUNK, plus: when a thread stores an owner into a place other threads can
                       reach (`brc_publish`: a lock-guarded `thread_shared` container, a channel, a spawned
                       thread's capture; all visible when compiling), it merges its own count of that object
                       at once, so other threads' later drops never go through its queue (fix 2).

   Biased counting:
     - `biased`: a count only the making thread reads or writes, with plain instructions.
     - `shared`: a locked 64-bit word: the other threads' units (signed) times 8, plus MERGED (the making
       thread gave up `biased`), QUEUED (the object waits in the making thread's queue), DEAD (ended).
     - The object lives while biased + count(shared) > 0.
     - The making thread's +1/-1 go to `biased` until the object is merged; when `biased` reaches 0 it
       merges (one compare-exchange; the object ends there if count(shared) is 0).
     - Another thread's +1/-1 go to `shared`. A -1 that takes count(shared) below 0 queues the object to
       the making thread, which merges it at its next safe point; a thread that has ended merges nothing,
       so a -1 meant for a closed queue drains that queue itself.
     - An opening through a `weakref`: on the making thread (not merged), alive if biased + count(shared)
       > 0, then biased += 1; on any other thread, one locked add to `shared`; DEAD means ended.
   All designs: the weak count starts at 1 for all owners together; the block is freed when it reaches 0
   ([OBJ-3]). The end of an object: its drop (the test's callback), then the owners' unit.

   Memory: every design takes blocks from per-thread 64 KiB chunks (a block freed on another thread goes
   back to its chunk's thread through a lock-free list, as mimalloc does), so the allocator is the same
   for all. The test build (BRC_CHECK) never reuses a block: it poisons it, so a read of a freed block is
   seen. BRC_COUNT_LOCKS counts every locked instruction. */
#pragma once
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <intrin.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(COUNT_BIASED) || defined(COUNT_BIASED_CHUNK) || defined(COUNT_BIASED_PUB)
#define BRC 1
#endif
#if defined(COUNT_BIASED_CHUNK) || defined(COUNT_BIASED_PUB)
#define BRC_CHUNK_OWNER 1
#endif
#if !defined(BRC) && !defined(COUNT_ATOMIC)
#error define COUNT_ATOMIC, COUNT_BIASED, COUNT_BIASED_CHUNK or COUNT_BIASED_PUB
#endif

#if defined(COUNT_ATOMIC)
#define DESIGN "atomic"
#elif defined(COUNT_BIASED)
#define DESIGN "biased"
#elif defined(COUNT_BIASED_CHUNK)
#define DESIGN "biased_chunk"
#else
#define DESIGN "biased_pub"
#endif

#define NOINLINE __declspec(noinline)
#if defined(__clang__)
#define UNLIKELY(c) __builtin_expect(!!(c), 0)
#else
#define UNLIKELY(c) (c)
#endif

static NOINLINE void brc_stop(const char *msg) {
    fflush(stdout);
    fprintf(stderr, "panic: %s\n", msg);
    exit(101);
}

#if defined(BRC_COUNT_LOCKS)
static __declspec(thread) long brc_locks;
#define LK (brc_locks++),
#else
#define LK
#endif
#define AT_ADD32(p, v) (LK (uint32_t)_InterlockedExchangeAdd((volatile long *)(p), (long)(v)))
#define AT_CAS32(p, e, w) (LK (uint32_t)_InterlockedCompareExchange((volatile long *)(p), (long)(w), (long)(e)))
#define AT_ADD64(p, v) (LK (int64_t)_InterlockedExchangeAdd64((volatile long long *)(p), (long long)(v)))
#define AT_CAS64(p, e, w) (LK (int64_t)_InterlockedCompareExchange64((volatile long long *)(p), (long long)(w), (long long)(e)))
#define AT_CASPTR(p, e, w) (LK _InterlockedCompareExchangePointer((void *volatile *)(p), (w), (e)))
#define AT_XCHGPTR(p, w) (LK _InterlockedExchangePointer((void *volatile *)(p), (w)))
#define AT_LOAD32(p) (*(volatile uint32_t *)(p))
#define AT_LOAD64(p) (*(volatile int64_t *)(p))
#define AT_STORE32(p, v) (*(volatile uint32_t *)(p) = (v))

typedef struct Obj Obj;
typedef void (*DropFn)(Obj *);

struct Obj {
#if defined(COUNT_ATOMIC)
    uint32_t count;            /* owners */
    uint32_t type_index;
    uint32_t weak;
    uint32_t pad;
#else
    uint32_t biased;           /* the making thread only, plain */
    uint32_t type_index;
    uint32_t weak;             /* locked */
    uint32_t owner;            /* COUNT_BIASED: the making thread's id, 0 once merged; else unused */
    volatile int64_t shared;   /* locked: count * 8 | DEAD | QUEUED | MERGED */
    Obj *queue_next;           /* the making thread's queue */
#endif
    volatile uint64_t canary;  /* LIVE while the object lives, ENDED after its drop, FREED after its free */
    uint64_t value;
    Obj *volatile child;       /* the test's one owner field: an object this one owns (its drop ends it) */
};

#define LIVE 0x4C4956454C495645ull
#define ENDED 0x454E444544454E44ull
#define FREED 0x4652454544465245ull

static DropFn brc_drop_fn;
static volatile long brc_freed_blocks, brc_made_objects;

#if defined(BRC_CHECK)
static volatile long pc_queued, pc_closed_drain, pc_merge_zero, pc_merge_drain, pc_merge_skip,
    pc_shared_open, pc_shared_open_dead, pc_owner_open, pc_owner_open_dead, pc_end_after_merge,
    pc_end_race_lost, pc_merging_skip, pc_publish_merge;
#define PC(x) _InterlockedIncrement(&x)
#else
#define PC(x) ((void)0)
#endif

/* ------------------------------------------------------------------ threads and the allocator */
#define CHUNK_SIZE (64 * 1024)
#define BLOCK_SIZE ((sizeof(Obj) + 15) & ~(size_t)15)
typedef struct { uint32_t owner; } ChunkHeader;   /* the first 64 bytes of each chunk; written once */

/* One thread's record. What only the thread itself writes and what other threads write (remote frees,
   queued objects) sit in separate 128-byte blocks, and each record starts on its own 128 bytes, so no two
   threads' records share a cache line (or the line the hardware fetches next to it). */
typedef struct {
    uint32_t id;                /* never reused */
    char *bump, *end;           /* the current chunk */
    void *local_free;           /* blocks freed on this thread */
    long locks;                 /* BRC_COUNT_LOCKS: this thread's locked instructions, at its end */
    char pad1[128 - 4 - 4 - 3 * 8 - 8];
    void *volatile remote_free; /* blocks of this thread's chunks freed on other threads */
    Obj *volatile queue;        /* biased counting: objects queued to this thread */
    volatile long closed;       /* set when the thread has ended */
    char pad2[128 - 2 * 8 - 8];
} BrcThread;

static volatile long brc_next_id = 1;
static __declspec(thread) BrcThread *brc_me;
static BrcThread **brc_threads;   /* id -> thread; sized by the program */

static inline ChunkHeader *chunk_of(const void *p) {
    return (ChunkHeader *)((uintptr_t)p & ~(uintptr_t)(CHUNK_SIZE - 1));
}

static BrcThread *brc_thread_start(void) {
    BrcThread *t = (BrcThread *)_aligned_malloc(sizeof(BrcThread), 128);
    if (!t) brc_stop("out of memory");
    memset(t, 0, sizeof(BrcThread));
    t->id = (uint32_t)_InterlockedIncrement(&brc_next_id);
    brc_threads[t->id] = t;
    brc_me = t;
    return t;
}

static void *block_alloc(BrcThread *t) {
    void *p = t->local_free;
    if (p) { t->local_free = *(void **)p; return p; }
    if (t->remote_free) {
        p = AT_XCHGPTR(&t->remote_free, NULL);
        t->local_free = *(void **)p;
        return p;
    }
    if (!t->bump || t->bump + BLOCK_SIZE > t->end) {
        char *c = (char *)VirtualAlloc(NULL, CHUNK_SIZE, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
        if (!c) brc_stop("out of memory");
        if ((uintptr_t)c & (CHUNK_SIZE - 1)) brc_stop("chunk not aligned");
        ((ChunkHeader *)c)->owner = t->id;
        t->bump = c + 64;
        t->end = c + CHUNK_SIZE;
    }
    p = t->bump;
    t->bump += BLOCK_SIZE;
    return p;
}
static void block_release(void *p) {
#if defined(BRC_CHECK)
    (void)p;                                        /* never reused: a later read of it is seen */
#else
    BrcThread *me = brc_me;
    uint32_t owner = chunk_of(p)->owner;
    if (owner == me->id) { *(void **)p = me->local_free; me->local_free = p; return; }
    BrcThread *o = brc_threads[owner];
    for (;;) {
        void *head = o->remote_free;
        *(void **)p = head;
        if (AT_CASPTR(&o->remote_free, head, p) == head) return;
    }
#endif
}

static NOINLINE void block_free(Obj *o) {
    if (o->canary != ENDED) brc_stop("a block freed that had not ended (or freed twice)");
    o->canary = FREED;
#if defined(BRC_CHECK)
    _InterlockedIncrement(&brc_freed_blocks);
#endif
    block_release(o);
}
static inline void weak_add(Obj *o) {
    if (UNLIKELY(AT_ADD32(&o->weak, 1) >= 0x7FFFFFFFu)) brc_stop("too many `weakref` links (2,147,483,647)");
}
static inline void weak_sub(Obj *o) {
    if (AT_ADD32(&o->weak, -1) == 1) block_free(o);
}
static NOINLINE void object_ended(Obj *o) {
    if (o->canary != LIVE) brc_stop("an object ended twice, or after its block was freed");
    if (brc_drop_fn) brc_drop_fn(o);
    o->canary = ENDED;
    weak_sub(o);                                    /* the owners' unit */
}

#if defined(COUNT_ATOMIC)
/* ------------------------------------------------------------------ today's rule */
static inline Obj *obj_new(BrcThread *t, uint64_t value) {
    Obj *o = (Obj *)block_alloc(t);
    o->count = 1; o->type_index = 7; o->weak = 1; o->pad = 0;
    o->canary = LIVE; o->value = value; o->child = NULL;
#if defined(BRC_CHECK)
    _InterlockedIncrement(&brc_made_objects);
#endif
    return o;
}
static inline void own_inc(BrcThread *t, Obj *o) {
    (void)t;
    if (UNLIKELY(AT_ADD32(&o->count, 1) >= 0x7FFFFFFFu)) brc_stop("too many owners (2,147,483,647)");
}
static inline void own_dec(BrcThread *t, Obj *o) {
    (void)t;
    if (AT_ADD32(&o->count, -1) == 1) object_ended(o);
}
static inline int weak_open(BrcThread *t, Obj *o) {
    (void)t;
    uint32_t c = AT_LOAD32(&o->count);
    for (;;) {
        if (c == 0) return 0;
        if (UNLIKELY(c >= 0x7FFFFFFFu)) brc_stop("too many owners (2,147,483,647)");
        uint32_t seen = AT_CAS32(&o->count, c, c + 1);
        if (seen == c) return 1;
        c = seen;
    }
}
static inline void view_end(BrcThread *t, Obj *o, int how) { (void)how; own_dec(t, o); }
static inline void brc_publish(BrcThread *t, Obj *o) { (void)t; (void)o; }
static inline void brc_safe_point(BrcThread *t) { (void)t; }
static void brc_thread_end(BrcThread *t) { (void)t; }

#else
/* ------------------------------------------------------------------ biased counting */
#define MERGED 1ll
#define QUEUED 2ll
#define DEAD 4ll
#define UNIT 8ll
#define COUNT(w) ((w) >> 3)   /* arithmetic shift: the signed count */

/* Is this object's count still biased to thread t? */
#if defined(BRC_CHUNK_OWNER)
static inline int biased_here(BrcThread *t, Obj *o) {
    return chunk_of(o)->owner == t->id && !(AT_LOAD64(&o->shared) & MERGED);
}
static inline uint32_t maker_of(Obj *o) { return chunk_of(o)->owner; }
#else
static inline int biased_here(BrcThread *t, Obj *o) { return o->owner == t->id; }
static inline uint32_t maker_of(Obj *o) { return AT_LOAD32(&o->owner); }
#endif

static inline Obj *obj_new(BrcThread *t, uint64_t value) {
    Obj *o = (Obj *)block_alloc(t);
    o->biased = 1; o->type_index = 7; o->weak = 1; o->owner = t->id; o->shared = 0; o->queue_next = NULL;
    o->canary = LIVE; o->value = value; o->child = NULL;
#if defined(BRC_CHECK)
    _InterlockedIncrement(&brc_made_objects);
#endif
    return o;
}

/* Rule Q: an object marked QUEUED is ended only by the drain that takes it off its queue. So once a
   thread has marked an object QUEUED (in the same atomic step as the drop that made its other-thread
   count negative), the object cannot end, and its block cannot be freed, until it has been pushed onto
   the queue and drained: the owners' weak unit keeps the block meanwhile. */

/* Merge `o` on its making thread (its `biased` reached 0, or it is being published): fold `biased` into
   `shared` and set MERGED. The object ends here only if the total is 0 and it is not QUEUED. */
static NOINLINE void merge(Obj *o) {
    int64_t add = (int64_t)o->biased * UNIT;
    o->biased = 0;
    int64_t w = AT_LOAD64(&o->shared);
    for (;;) {
        int64_t nw = (w + add) | MERGED;
        int ends = COUNT(nw) == 0 && !(nw & QUEUED);
        if (ends) nw |= DEAD;
        int64_t seen = AT_CAS64(&o->shared, w, nw);
        if (seen == w) {
#if !defined(BRC_CHUNK_OWNER)
            AT_STORE32(&o->owner, 0);               /* after the merge, so a QUEUED mark always finds the maker */
#endif
            if (ends) object_ended(o);
            return;
        }
        w = seen;
    }
}

/* Take `o` off its queue (on the making thread, or on any thread once the maker has ended): merge it if it
   is not merged yet, clear QUEUED, and end it if its total is 0 (rule Q). */
static NOINLINE void unqueue(Obj *o) {
    int64_t add = 0;
    if (!(AT_LOAD64(&o->shared) & MERGED)) {        /* only this thread can merge it now: `biased` is stable */
        add = (int64_t)o->biased * UNIT;
        o->biased = 0;
        PC(pc_merge_drain);
    } else {
        PC(pc_merge_skip);
    }
    int64_t w = AT_LOAD64(&o->shared);
    for (;;) {
        int64_t nw = ((w + add) | MERGED) & ~QUEUED;
        int ends = COUNT(nw) == 0;
        if (ends) nw |= DEAD;
        int64_t seen = AT_CAS64(&o->shared, w, nw);
        if (seen == w) {
#if !defined(BRC_CHUNK_OWNER)
            AT_STORE32(&o->owner, 0);
#endif
            if (ends) object_ended(o);
            return;
        }
        w = seen;
    }
}

static NOINLINE void drain_queue(BrcThread *t) {
    Obj *q = (Obj *)AT_XCHGPTR(&t->queue, NULL);
    while (q) {
        Obj *next = q->queue_next;                  /* read first: unqueue may free q */
        unqueue(q);
        q = next;
    }
}

static NOINLINE void queue_to_owner(Obj *o, uint32_t owner) {
    BrcThread *t = brc_threads[owner];
    for (;;) {
        Obj *head = t->queue;
        o->queue_next = head;
        if (AT_CASPTR(&t->queue, head, o) == head) break;
    }
    PC(pc_queued);
    if (t->closed) { PC(pc_closed_drain); drain_queue(t); }   /* the maker has ended: nobody else will drain it */
}

/* Another thread's -1, in one atomic step: it marks QUEUED when it makes the count negative before the
   merge, and it ends the object when it takes the last unit after the merge (unless QUEUED: rule Q). */
static inline void shared_sub(Obj *o) {
    int64_t w = AT_LOAD64(&o->shared);
#if defined(SHARED_SUB_FAST)
    /* Once MERGED is seen it stays set, so no QUEUED mark can be needed any more: one locked add, then,
       if that took the last unit and the object is not QUEUED (rule Q), mark it DEAD; an opening that
       came in between makes that exchange fail, and its own -1 ends the object instead. */
    if (w & MERGED) {
        int64_t nw = AT_ADD64(&o->shared, -UNIT) - UNIT;
        if (COUNT(nw) == 0 && !(nw & (QUEUED | DEAD))) {
            if (AT_CAS64(&o->shared, nw, nw | DEAD) == nw) { PC(pc_end_after_merge); object_ended(o); }
            else PC(pc_end_race_lost);
        }
        return;
    }
#endif
    for (;;) {
        int64_t nw = w - UNIT;
        int queue = COUNT(nw) < 0 && !(nw & (MERGED | QUEUED));
        if (queue) nw |= QUEUED;
        int ends = (nw & MERGED) && !(nw & QUEUED) && COUNT(nw) == 0;
        if (ends) nw |= DEAD;
        int64_t seen = AT_CAS64(&o->shared, w, nw);
        if (seen == w) {
            if (ends) { PC(pc_end_after_merge); object_ended(o); }
            else if (queue) queue_to_owner(o, maker_of(o));
            return;
        }
        PC(pc_end_race_lost);
        w = seen;
    }
}

static inline void own_inc(BrcThread *t, Obj *o) {
    if (biased_here(t, o)) {
        if (UNLIKELY(o->biased == 0xFFFFFFFFu)) brc_stop("too many owners (4,294,967,295)");
        o->biased += 1;
    } else {
        int64_t w = AT_ADD64(&o->shared, UNIT);
        if (UNLIKELY(COUNT(w) >= 0x7FFFFFFF)) brc_stop("too many owners (2,147,483,647)");
    }
}
static inline void own_dec(BrcThread *t, Obj *o) {
    if (biased_here(t, o)) {
        if (--o->biased == 0) { PC(pc_merge_zero); merge(o); }
    } else {
        shared_sub(o);
    }
}
/* Returns 0 (ended), 1 (the unit is in `biased`) or 2 (in `shared`). */
static inline int weak_open(BrcThread *t, Obj *o) {
    if (biased_here(t, o)) {
        int64_t w = AT_LOAD64(&o->shared);
        if ((int64_t)o->biased + COUNT(w) <= 0) { PC(pc_owner_open_dead); return 0; }
        if (UNLIKELY(o->biased == 0xFFFFFFFFu)) brc_stop("too many owners (4,294,967,295)");
        PC(pc_owner_open);
        o->biased += 1;
        return 1;
    }
    int64_t w = AT_ADD64(&o->shared, UNIT);
    if (w & DEAD) { PC(pc_shared_open_dead); AT_ADD64(&o->shared, -UNIT); return 0; }
    if (UNLIKELY(COUNT(w) >= 0x7FFFFFFF)) brc_stop("too many owners (2,147,483,647)");
    PC(pc_shared_open);
    return 2;
}
static inline void view_end(BrcThread *t, Obj *o, int how) {
    (void)t;
    if (how == 1) {
        if (--o->biased == 0) { PC(pc_merge_zero); merge(o); }
        return;
    }
    shared_sub(o);
}
/* An owner of `o` is being stored where other threads can reach it. */
static inline void brc_publish(BrcThread *t, Obj *o) {
#if defined(COUNT_BIASED_PUB)
    if (biased_here(t, o)) { PC(pc_publish_merge); merge(o); }
#else
    (void)t; (void)o;
#endif
}
static inline void brc_safe_point(BrcThread *t) {
    if (t->queue) drain_queue(t);
}
static void brc_thread_end(BrcThread *t) {
    _InterlockedExchange(&t->closed, 1);
    drain_queue(t);
}
#endif
