/* counts.h: the Player object and its owner count, one form per VARIANT, shared by the two owner-count
   workloads (retarget_body.h, hotshare_body.h) so that every program of a workload does the same work
   in the same order and differs only in what a `shares` owner costs.

   The Ember program both workloads model (spec 0.9.15_Hardened_1):
       class Player:                      # a class object: its own block, 8-byte header ([OBJ-1])
           hp: int32
           x: int32
           inventory: Array[int32]
       struct Watcher:
           target: shares mut Player      # the watcher keeps its player alive (R9, [RC-1])
       players: Array[Player]             # the first owner of each player
       ws[i].target = shares mut players[r]   # re-point: +1 on the new target, -1 on the old one

   VARIANT
     V_PLAIN    plain C: no header, a raw pointer per watcher, no count (32-byte player)
     V_MODEL    the rule as written: 8-byte header {u32 count, u32 type index} ([OBJ-1]); before each +1
                the count is compared with its maximum 4,294,967,295 and the program stops there
                ([RC-1], [RT-7], R9); -1 and a test for zero, the end out of line ([RT-10])
     V_NOCMP    diagnostic: V_MODEL without the limit compare: a C++ intrusive count, the baseline of
                [BEN-6]'s gate ("code with `shares` owners <= 1.15x a C++ intrusive count"); also what
                run 3's cost-table row says it measured (see the report: run 3's MODEL 3 in fact had a
                two-sided compare)
     V_FOLD     salvage: the same compare, emitted as the increment's own result test ("count+1 == 0"),
                so the add and the branch fuse; same stop, same limit, same 8-byte header
     V_WIDE     salvage: one 64-bit header word, type index in bits 0..18, (owners - 1) in bits 19..63
                (45 bits); +1 is one add with no compare, -1 is one subtract whose borrow says "last
                owner"; same 8 bytes. Safety argument in the report (owners are bounded by memory)
     V_TS       `thread_shared Player` as written: the count is atomic, a +1 is one atomic add and the
                limit is tested on the value it returns (stop at 2,147,483,647 or more, half the range,
                [RC-4], [RT-7], [RT-10]); -1 an atomic subtract ([RT-8])
     V_TS_WIDE  salvage of V_TS: the V_WIDE word, raised with one locked add and no limit test
     V_TS_NOCMP diagnostic: V_TS without the limit test, a C++ intrusive atomic count (`lock inc`):
                [BEN-6]'s baseline for `thread_shared` ("or atomic for `thread_shared`")

   For V_TS and V_TS_WIDE the bodies keep changing `hp` and `inventory` directly, which [RC-7] / E7014
   refuse for a `thread_shared` value (it changes only through Atomic / Mutex / a guarded field): the
   body is kept identical so the difference to V_MODEL is the count alone; the lock a real program takes
   is the "a lock" row's cost, not this one. hotshare_body.h only reads `x`, which is legal as is. */
#pragma once
#include "m1rt.h"

#ifndef VARIANT
#error define VARIANT
#endif
#define V_PLAIN 0
#define V_MODEL 1
#define V_NOCMP 2
#define V_FOLD 3
#define V_WIDE 4
#define V_TS 5
#define V_TS_WIDE 6
#define V_TS_NOCMP 7

typedef struct { int32_t *p; size_t len, cap; } IVec;
static NOINLINE void ivec_grow(IVec *v) {
    size_t cap = v->cap ? v->cap * 2 : 4;
    int32_t *p = (int32_t *)realloc(v->p, cap * sizeof(int32_t));
    if (!p) abort();
    v->p = p; v->cap = cap;
}
static inline void ivec_push(IVec *v, int32_t x) { if (v->len == v->cap) ivec_grow(v); v->p[v->len++] = x; }

#define PLAYER_TYPE_INDEX 7u

#if VARIANT == V_PLAIN
typedef struct Player { int32_t hp; int32_t x; IVec inv; } Player;                      /* 32 bytes */
#elif VARIANT == V_WIDE || VARIANT == V_TS_WIDE
#define WIDE_ONE (1ull << 19)               /* one owner, in the word's bits 19..63 */
typedef struct Player { uint64_t word; int32_t hp; int32_t x; IVec inv; } Player;       /* 40 bytes */
#else
typedef struct { uint32_t count, type_index; } ObjHeader;                               /* [OBJ-1] */
typedef struct Player { ObjHeader h; int32_t hp; int32_t x; IVec inv; } Player;         /* 40 bytes */
#endif

#if VARIANT != V_PLAIN
/* The count reached zero: the object ends ([RC-1]); out of line ([RT-10]). Never reached in these
   workloads (`players` keeps one owner of every player); written so the release is the real one. */
static NOINLINE void player_end(Player *o) { free(o->inv.p); o->inv.p = NULL; o->inv.len = o->inv.cap = 0; }
#endif

static inline void player_init_header(Player *p) {
#if VARIANT == V_WIDE || VARIANT == V_TS_WIDE
    p->word = PLAYER_TYPE_INDEX;            /* owners - 1 = 0: one owner */
#elif VARIANT != V_PLAIN
    p->h.count = 1; p->h.type_index = PLAYER_TYPE_INDEX;
#else
    (void)p;
#endif
}

/* RETAIN: a `shares mut` owner made. */
static inline void retain(Player *o) {
#if VARIANT == V_MODEL
    if (UNLIKELY(o->h.count == 0xFFFFFFFFu))                       /* [RC-1] [RT-7] R9: compare before +1 */
        e_stop("too many owners of a `Player` (4,294,967,295)");
    o->h.count += 1;                                               /* [RC-1] [RT-8]: a plain add */
#elif VARIANT == V_NOCMP
    o->h.count += 1;                                               /* diag: no limit compare */
#elif VARIANT == V_FOLD
    uint32_t n = o->h.count + 1;                                   /* [RC-1] [RT-7]: the same compare, */
    if (UNLIKELY(n == 0))                                          /* read from the add's result; the stop */
        e_stop("too many owners of a `Player` (4,294,967,295)");   /* runs before the wrapped value is stored */
    o->h.count = n;
#elif VARIANT == V_WIDE
    o->word += WIDE_ONE;                                           /* salvage: no compare (owners < 2^44) */
#elif VARIANT == V_TS
    uint32_t before = AT_ADD32(&o->h.count, 1);                    /* [RT-8] [RT-10]: one atomic add */
    if (UNLIKELY(before >= 0x7FFFFFFFu))                           /* [RC-4] [RT-7]: limit on the returned value */
        e_stop("too many owners of a `Player` (2,147,483,647)");
#elif VARIANT == V_TS_WIDE
    (void)AT_ADD64(&o->word, WIDE_ONE);                            /* salvage: one locked add, no test */
#elif VARIANT == V_TS_NOCMP
    (void)AT_ADD32(&o->h.count, 1);                                /* diag: one locked add, no limit test */
#else
    (void)o;
#endif
}

/* RELEASE: a `shares mut` owner ended ([RC-1]: the subtraction that reaches zero ends the value). */
static inline void release(Player *o) {
#if VARIANT == V_MODEL || VARIANT == V_NOCMP || VARIANT == V_FOLD
    if (--o->h.count == 0) player_end(o);                          /* [RC-1] [RT-10] */
#elif VARIANT == V_WIDE
    uint64_t before = o->word;
    o->word = before - WIDE_ONE;
    if (before < WIDE_ONE) player_end(o);                          /* borrow: that was the last owner */
#elif VARIANT == V_TS || VARIANT == V_TS_NOCMP
    if (AT_ADD32(&o->h.count, -1) == 1) player_end(o);            /* [RT-8]: acquire-release decrement */
#elif VARIANT == V_TS_WIDE
    if (AT_ADD64(&o->word, (uint64_t)0 - WIDE_ONE) < WIDE_ONE) player_end(o);
#else
    (void)o;
#endif
}

/* The program's name in the output line, from the build's NAME. */
#ifndef NAME
#error define NAME
#endif
