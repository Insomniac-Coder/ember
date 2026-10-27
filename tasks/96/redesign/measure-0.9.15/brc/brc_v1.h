/* brc.h: two ways to count the owners of a `thread_shared` object, behind one interface, so that one
   stress test and one set of timed patterns can run on either (the lead's, 2026-10-27).

   COUNT_ATOMIC  today's rule (0.9.15: [RC-4], [RC-5] point 2, [WK-12], [RT-8], [RT-10]): one owner count,
                 every +1 and -1 a locked instruction; a `weakref` opening is a compare-exchange loop that
                 raises the count only while it is above 0; the weak count is locked too.
   COUNT_BIASED  biased reference counting (Choi, Shull and Torrellas, PACT 2018), adapted:
     - `owner`: the runtime id of the thread that made the object (never reused; 0 = no owner thread).
     - `biased`: a count only the owner thread ever reads or writes, with plain instructions.
     - `shared`: a locked 64-bit word: the count of the other threads' units (signed) times 8, plus three
       flags: MERGED (the owner thread gave up `biased`: every thread now uses `shared`), QUEUED (the
       object is in its owner thread's queue), DEAD (the object has ended).
     - The object lives while biased + count(shared) > 0.
     - The owner thread's +1/-1 go to `biased`. When `biased` reaches 0 it merges: one compare-exchange
       sets MERGED, and DEAD as well if count(shared) is 0 (then the object ends, at once, on that thread).
     - Another thread's +1/-1 go to `shared`. A -1 that takes count(shared) below 0 (that thread dropped
       an owner the owner thread counted) queues the object to the owner thread, which merges it at its
       next safe point (`brc_safe_point`): biased is folded into shared, MERGED set, and the object ends
       there if the total is 0. So such an object's end can come later, on its owner thread.
     - A thread that ends merges its queue and closes it; a -1 meant for a closed queue merges at once
       (the owner thread is gone, so `biased` can no longer change).
     - An opening through a `weakref` on the owner thread: alive if biased + count(shared) > 0, then
       biased += 1. On another thread: count(shared) += 1 with one locked add; DEAD means ended (the 1 is
       given back); otherwise the unit now held keeps the object alive.
   Both: the weak count starts at 1 for all owners together; the block is freed when it reaches 0
   ([OBJ-3]). The end of an object: its drop (here: a callback the test sets), then the owners' unit.

   The test build (BRC_CHECK) never frees a block: it poisons it, so a read of a freed block is seen. */
#pragma once
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <intrin.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

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

#define AT_ADD32(p, v) ((uint32_t)_InterlockedExchangeAdd((volatile long *)(p), (long)(v)))
#define AT_CAS32(p, e, w) ((uint32_t)_InterlockedCompareExchange((volatile long *)(p), (long)(w), (long)(e)))
#define AT_ADD64(p, v) ((int64_t)_InterlockedExchangeAdd64((volatile long long *)(p), (long long)(v)))
#define AT_CAS64(p, e, w) ((int64_t)_InterlockedCompareExchange64((volatile long long *)(p), (long long)(w), (long long)(e)))
#define AT_LOAD32(p) (*(volatile uint32_t *)(p))
#define AT_LOAD64(p) (*(volatile int64_t *)(p))
#define AT_STORE32(p, v) (*(volatile uint32_t *)(p) = (v))

typedef struct Obj Obj;
typedef void (*DropFn)(Obj *);

/* The header, then the test's fields. */
struct Obj {
#if defined(COUNT_ATOMIC)
    uint32_t count;            /* owners; bit 31 unused: the CAS loop never raises it from 0 */
    uint32_t type_index;
    uint32_t weak;
    uint32_t pad;
#else
    uint32_t biased;           /* owner thread only, plain */
    uint32_t type_index;
    uint32_t weak;             /* locked */
    uint32_t owner;            /* the owner thread's id; written by it alone (to 0 when it merges) */
    volatile int64_t shared;   /* locked: count * 8 | DEAD | QUEUED | MERGED */
    Obj *queue_next;           /* the owner thread's queue (written under QUEUED by one thread) */
#endif
    /* the test's payload */
    volatile uint64_t canary;  /* LIVE while the object lives, ENDED after its drop, FREED after its free */
    uint64_t value;
};

#define LIVE 0x4C4956454C495645ull
#define ENDED 0x454E444544454E44ull
#define FREED 0x4652454544465245ull

static DropFn brc_drop_fn;    /* the test's drop: called once per object, when it ends */
/* Path counters (the test build only): proof that the stress test reached each path. */
#if defined(BRC_CHECK)
static volatile long pc_queued, pc_closed_drain, pc_merge_zero, pc_merge_drain, pc_merge_skip,
    pc_shared_open, pc_shared_open_dead, pc_owner_open, pc_owner_open_dead, pc_end_after_merge,
    pc_end_race_lost, pc_merging_skip;
#define PC(x) _InterlockedIncrement(&x)
#else
#define PC(x) ((void)0)
#endif
static volatile long brc_freed_blocks, brc_made_objects;

static NOINLINE void block_free(Obj *o) {
    if (o->canary != ENDED) brc_stop("a block freed that had not ended (or freed twice)");
    o->canary = FREED;
    _InterlockedIncrement(&brc_freed_blocks);
#if !defined(BRC_CHECK)
    free(o);
#endif
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
    weak_sub(o);                                   /* the owners' unit */
}

/* ------------------------------------------------------------------ threads */
typedef struct {
    uint32_t id;               /* never reused */
#if !defined(COUNT_ATOMIC)
    Obj *queue;                /* objects queued to this thread (pushed by others, lock-free stack) */
    volatile long closed;      /* set when the thread has ended */
#endif
} BrcThread;

static volatile long brc_next_id = 1;
static __declspec(thread) BrcThread *brc_me;

/* Every thread calls brc_thread_start first. A real compiler would load the id once per function. */
static BrcThread *brc_thread_start(void) {
    BrcThread *t = (BrcThread *)calloc(1, sizeof(BrcThread));
    if (!t) brc_stop("out of memory");
    t->id = (uint32_t)_InterlockedIncrement(&brc_next_id);
    brc_me = t;
    return t;
}

#if defined(COUNT_ATOMIC)
/* ------------------------------------------------------------------ today's rule */
static inline Obj *obj_new(BrcThread *t, uint64_t value) {
    (void)t;
    Obj *o = (Obj *)malloc(sizeof(Obj));
    if (!o) brc_stop("out of memory");
    o->count = 1; o->type_index = 7; o->weak = 1; o->pad = 0;
    o->canary = LIVE; o->value = value;
    _InterlockedIncrement(&brc_made_objects);
    return o;
}
/* The caller already holds a unit (an owner it copies). */
static inline void own_inc(BrcThread *t, Obj *o) {
    (void)t;
    if (UNLIKELY(AT_ADD32(&o->count, 1) >= 0x7FFFFFFFu)) brc_stop("too many owners (2,147,483,647)");
}
static inline void own_dec(BrcThread *t, Obj *o) {
    (void)t;
    if (AT_ADD32(&o->count, -1) == 1) object_ended(o);
}
/* An opening through a `weakref` link: returns 1 and holds a unit, or 0 if the object ended. */
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
static inline void brc_safe_point(BrcThread *t) { (void)t; }
static inline void brc_thread_end(BrcThread *t) { (void)t; }

#else
/* ------------------------------------------------------------------ biased counting */
#define MERGED 1ll
#define QUEUED 2ll
#define DEAD 4ll
#define UNIT 8ll
#define COUNT(w) ((w) >> 3)   /* arithmetic shift: the signed count */

static BrcThread **brc_threads;      /* id -> thread, for queueing (the test sizes it) */

static inline Obj *obj_new(BrcThread *t, uint64_t value) {
    Obj *o = (Obj *)malloc(sizeof(Obj));
    if (!o) brc_stop("out of memory");
    o->biased = 1; o->type_index = 7; o->weak = 1; o->owner = t->id; o->shared = 0; o->queue_next = NULL;
    o->canary = LIVE; o->value = value;
    _InterlockedIncrement(&brc_made_objects);
    return o;
}

/* Merge `o`: fold `biased` into `shared`, set MERGED, end the object if the total is 0. Called by the
   owner thread (at a safe point, or when biased reaches 0), or by the one thread that queued it after
   the owner thread ended. `biased` cannot change meanwhile: only the owner thread writes it. */
static NOINLINE void merge(Obj *o) {
    int64_t add = (int64_t)o->biased * UNIT;
    o->biased = 0;
    AT_STORE32(&o->owner, 0);                       /* every thread now uses `shared` */
    int64_t w = AT_LOAD64(&o->shared);
    for (;;) {
        int64_t nw = (w + add) | MERGED;
        nw &= ~QUEUED;
        if (COUNT(nw) == 0) nw |= DEAD;
        int64_t seen = AT_CAS64(&o->shared, w, nw);
        if (seen == w) { if (COUNT(nw) == 0) object_ended(o); return; }
        w = seen;
    }
}

/* Take the whole queue of `t` and merge each object once. The owner thread does this at its safe points;
   any thread does it for a queue whose thread has ended. An object already merged (its `biased` reached 0
   while it waited) is skipped. The queue held a weak unit on each object, so no block is freed while
   it waits; that unit is given back here. */
static NOINLINE void drain_queue(BrcThread *t) {
    Obj *q = (Obj *)_InterlockedExchangePointer((void *volatile *)&t->queue, NULL);
    while (q) {
        Obj *next = q->queue_next;
        if (!(AT_LOAD64(&q->shared) & MERGED)) { PC(pc_merge_drain); merge(q); } else PC(pc_merge_skip);
        weak_sub(q);
        q = next;
    }
}

/* Queue `o` to its owner thread (the caller set QUEUED, so it is the only one doing so). */
static NOINLINE void queue_to_owner(Obj *o, uint32_t owner) {
    BrcThread *t = brc_threads[owner];
    weak_add(o);                                    /* the queue keeps the block */
    for (;;) {
        Obj *head = (Obj *)t->queue;
        o->queue_next = head;
        if (_InterlockedCompareExchangePointer((void *volatile *)&t->queue, o, head) == head) break;
    }
    /* If the owner thread has ended (it set `closed` before its last drain), nobody else will drain
       this queue: drain it here. The interlocked push and the owner's interlocked close order the two,
       so an object pushed after the owner's last drain always sees `closed`. */
    PC(pc_queued);
    if (t->closed) { PC(pc_closed_drain); drain_queue(t); }
}

static NOINLINE void shared_dec_slow(Obj *o, int64_t nw) {
    if ((nw & MERGED) && COUNT(nw) == 0) {          /* the last unit after the merge */
        if (AT_CAS64(&o->shared, nw, nw | DEAD) == nw) { PC(pc_end_after_merge); object_ended(o); }
        else PC(pc_end_race_lost);
        return;                                     /* else an opening came in first: its -1 retries */
    }
    if (!(nw & MERGED) && COUNT(nw) < 0 && !(nw & QUEUED)) {
        int64_t w = nw;
        for (;;) {                                  /* set QUEUED once */
            if (w & (QUEUED | MERGED)) return;
            int64_t seen = AT_CAS64(&o->shared, w, w | QUEUED);
            if (seen == w) break;
            w = seen;
        }
        uint32_t owner = AT_LOAD32(&o->owner);
        if (owner == 0) { PC(pc_merging_skip); return; }   /* a merge is under way (it wrote owner = 0 before its
                                                       compare-exchange): that exchange sees this -1 and
                                                       clears QUEUED, so nothing is left to do */
        queue_to_owner(o, owner);
    }
}

static inline void own_inc(BrcThread *t, Obj *o) {
    if (o->owner == t->id) {
        if (UNLIKELY(o->biased == 0xFFFFFFFFu)) brc_stop("too many owners (4,294,967,295)");
        o->biased += 1;
    } else {
        int64_t w = AT_ADD64(&o->shared, UNIT);
        if (UNLIKELY(COUNT(w) >= 0x7FFFFFFF)) brc_stop("too many owners (2,147,483,647)");
    }
}
static inline void own_dec(BrcThread *t, Obj *o) {
    if (o->owner == t->id) {
        if (--o->biased == 0) { PC(pc_merge_zero); merge(o); }
    } else {
        int64_t nw = AT_ADD64(&o->shared, -UNIT) - UNIT;
        if (UNLIKELY(COUNT(nw) <= 0)) shared_dec_slow(o, nw);
    }
}
/* Returns 0 (ended), 1 (the unit is in `biased`) or 2 (in `shared`). */
static inline int weak_open(BrcThread *t, Obj *o) {
    if (o->owner == t->id) {
        int64_t w = AT_LOAD64(&o->shared);
        if ((int64_t)o->biased + COUNT(w) <= 0) { PC(pc_owner_open_dead); return 0; }   /* its owners have all ended */
        PC(pc_owner_open);
        if (UNLIKELY(o->biased == 0xFFFFFFFFu)) brc_stop("too many owners (4,294,967,295)");
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
    if (how == 1) {
        if (--o->biased == 0) { PC(pc_merge_zero); merge(o); }
        return;
    }
    (void)t;
    int64_t nw = AT_ADD64(&o->shared, -UNIT) - UNIT;
    if (UNLIKELY(COUNT(nw) <= 0)) shared_dec_slow(o, nw);
}
/* The owner thread merges what other threads queued to it. */
static inline void brc_safe_point(BrcThread *t) {
    if (t->queue) drain_queue(t);
}
static void brc_thread_end(BrcThread *t) {
    _InterlockedExchange(&t->closed, 1);
    drain_queue(t);                                  /* what was queued before the close */
    /* Objects this thread still owns (biased > 0) keep owner = t->id; any later -1 from another
       thread that goes below 0 finds the queue closed and merges at once (biased is frozen now). */
}
#endif
