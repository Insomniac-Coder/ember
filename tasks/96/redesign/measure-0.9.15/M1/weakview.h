/* weakview.h: a view of a `thread_shared` object opened from a `weakref` link (R10 point 2, [RC-5]
   point 2, [WK-12], [RT-10]). One file, one VARIANT per program, so that every program does the same
   work in the same order and differs only in how a link is kept, opened, re-pointed and ended.

   The Ember program (spec 0.9.15_Hardened_1):
       class Player:                                  # x and id never change after the player is made
           x: int
           id: int
       class Camera:
           target: weakref Player                     # [WK-11]: one weak count on the target
       players: Array[thread_shared Player]           # each player's one owner ([RC-1], [RC-4])
       cams: Array[Camera]                            # 1,048,576 cameras, camera i follows slot s(i)
       for f in 0..64:
           r = f % 64
           for j in range(r, len(players), 64):       # 256 players end and are replaced
               players[j] = thread_shared Player(x=j % 1024, id=len(players) * (f + 1) + j)
           for i in 0..len(cams):
               s = (i * 40503) & (len(players) - 1)
               if cams[i].target exists as p:          # opened: [WK-12]; R10 point 2: one atomic count
                   acc += p.x                          #   held for p's life ([RC-5] point 2)
               else:
                   cams[i].target = weakref players[s]  # the slot's new player: weak +1 / -1 ([WK-11])
                   fails += 1
       checksum = acc + fails

   16,384 slots, 1,048,576 cameras, 64 frames: 67,108,864 openings, 1,048,576 of which find the player
   ended. Checksum, worked out without running: 40503 is odd, so s(i) takes each of the 16,384 slots 64
   times; in frame f the cameras on the 256 slots j = r (mod 64) fail and the others add j % 1024, so
   frame f adds 64 * (8,257,536 - 256 r) = 528,482,304 - 16,384 r; over r = 0..63 that is
   33,789,837,312, and the failures add 64 * 16,384 = 1,048,576: checksum 33,790,885,888.

   VARIANT
     W_PLAIN      plain C as a careful game programmer writes it: the players inline in an array of
                  slots with a generation number; a camera keeps {slot, generation}; opening compares
                  the generations; a player's end bumps the slot's generation and reuses the slot in
                  place. No counts, no allocation, no checks.
     W_ONE        the same Ember program with `players: Array[Player]` (one thread, R11): a 12-byte
                  header {count, type index, weak count} ([OBJ-1]); opening tests the header only
                  ([WK-12]); no count for the view (R10 point 1); weak +1 compared with 4,294,967,295
                  ([WK-11], [RT-7]); the one-owner player is not counted, its end marks the count 0.
     W_TS         the rule as written: `thread_shared Player`; the opening is a compare-exchange loop
                  that adds one to the owner count only while it is above zero, with the limit test,
                  and the view gives that count back when it ends ([WK-12], [RC-5] point 2, [RT-10],
                  [RC-4]); the owner count and the weak count are atomic ([RT-8]).
     W_PLAINDATA  salvage of W_TS for a class whose end frees nothing and runs nothing (no `drop`, no
                  field that owns memory: "plain data"): the opening is an atomic load and a test, no
                  count; the block is kept by the link the view was opened from. Safety argument in
                  the report. */
#pragma once
#include "m1rt.h"

#ifndef VARIANT
#error define VARIANT
#endif
#ifndef NAME
#error define NAME
#endif
#define W_PLAIN 0
#define W_ONE 1
#define W_TS 2
#define W_PLAINDATA 3

#define WP (1u << 14)
#define WC (1u << 20)
#define WF 64u
#define WMULT 40503u
#define PLAYER_TYPE_INDEX 7u

#if VARIANT == W_PLAIN
/* ------------------------------------------------------------------ plain C: generational slots */
typedef struct { int64_t x, id; } Player;
typedef struct { uint32_t gen; uint32_t pad; Player p; } Slot;                           /* 24 bytes */
typedef struct { uint32_t slot, gen; } Link;                                             /* 8 bytes */
static Slot *slots;
static void world_init(void) {
    slots = (Slot *)malloc(WP * sizeof(Slot));
    if (!slots) abort();
    for (uint32_t j = 0; j < WP; j++) { slots[j].gen = 0; slots[j].pad = 0; slots[j].p.x = j & 1023; slots[j].p.id = j; }
}
static inline void link_set(Link *l, uint32_t s) { l->slot = s; l->gen = slots[s].gen; }
static inline void link_retarget(Link *l, uint32_t s) { link_set(l, s); }
static inline const Player *link_open(const Link *l) {
    const Slot *sl = &slots[l->slot];
    return sl->gen == l->gen ? &sl->p : NULL;
}
static inline void view_end(const Player *p) { (void)p; }
static inline void replace_player(uint32_t j, int64_t id) {
    slots[j].gen++; slots[j].p.x = j & 1023; slots[j].p.id = id;
}

#else
/* ------------------------------------------------------------------ Ember: objects in blocks */
typedef struct { uint32_t count, type_index, weak, pad; } ObjHeader;   /* [OBJ-1]: 12 bytes, then the fields' alignment */
typedef struct Player { ObjHeader h; int64_t x, id; } Player;          /* 32 bytes */
typedef struct { Player *target; } Link;                               /* [OBJ-2]: one pointer */
static Slab pslab = {0, 0, 0, sizeof(Player)};
static Player **players;                                                /* Array[(thread_shared) Player] */
static volatile size_t vlen = WP;
static size_t players_len;                                              /* not known when compiling */

static Player *new_player(uint32_t j, int64_t id) {
    Player *p = (Player *)slab_alloc(&pslab);
    p->h.count = 1; p->h.type_index = PLAYER_TYPE_INDEX; p->h.pad = 0;
#if VARIANT == W_ONE
    p->h.weak = 0;                                   /* [WK-11]: the number of links */
#else
    p->h.weak = 1;                                   /* the links, plus one unit the owners hold together
                                                        while the object lives (see the report: without it
                                                        the last owner's end and the last link's end race
                                                        to free the block; Rust's Arc does the same) */
#endif
    p->x = j & 1023; p->id = id;
    return p;
}

#if VARIANT == W_ONE
static inline void weak_add(Player *p) {
    if (UNLIKELY(p->h.weak == 0xFFFFFFFFu))                            /* [WK-11] [RT-7]: compare before +1 */
        e_stop("too many `weakref` links to a `Player` (4,294,967,295)");
    p->h.weak += 1;
}
static inline void weak_sub(Player *p) {
    if (--p->h.weak == 0 && p->h.count == 0) slab_free(&pslab, p);    /* the block goes with the last link */
}
static NOINLINE void owner_end(Player *p) {                            /* players[j] replaced: the one owner ends */
    p->h.count = 0;                                                     /* [RT-10]: marked ended, then its drop (none) */
    if (p->h.weak == 0) slab_free(&pslab, p);
}
#else
static NOINLINE void block_free(Player *p) { slab_free(&pslab, p); }
static inline void weak_add(Player *p) {
    if (UNLIKELY(AT_ADD32(&p->h.weak, 1) >= 0x7FFFFFFFu))              /* [RC-4] [RT-7] [RT-10]: one atomic add, */
        e_stop("too many `weakref` links to a `Player` (2,147,483,647)");  /* the limit tested on its result */
}
static inline void weak_sub(Player *p) {
    if (AT_ADD32(&p->h.weak, -1) == 1) block_free(p);                  /* [RT-8]: acquire-release decrement */
}
static NOINLINE void owner_end_slow(Player *p) {                      /* the count reached zero: the object ended */
    /* its drop: none; then the owners' unit of the weak count */
    weak_sub(p);
}
static inline void owner_sub(Player *p) {
    if (AT_ADD32(&p->h.count, -1) == 1) owner_end_slow(p);             /* [RT-8] */
}
static inline void owner_end(Player *p) { owner_sub(p); }              /* players[j] replaced: one owner ends */
#endif

static void world_init(void) {
    players = (Player **)malloc(WP * sizeof(Player *));
    if (!players) abort();
    for (uint32_t j = 0; j < WP; j++) players[j] = new_player(j, j);
    players_len = vlen;
}
static inline void link_set(Link *l, uint32_t s) {                     /* cams[i].target = weakref players[s] */
    if (UNLIKELY(s >= players_len)) e_stop("index out of bounds");     /* bounds check, as today */
    Player *n = players[s];
    weak_add(n);                                                        /* [WK-11] +1 on the new target */
    l->target = n;
}
static inline void link_retarget(Link *l, uint32_t s) {
    Player *old = l->target;
    link_set(l, s);
    if (old) weak_sub(old);                                             /* [WK-11] -1 on the old target */
}

static inline const Player *link_open(const Link *l) {                 /* if cams[i].target exists as p: */
    Player *p = l->target;
    if (p == NULL) return NULL;                                         /* [WK-11]: a link may hold no target */
#if VARIANT == W_ONE
    return p->h.count != 0 ? p : NULL;                                  /* [WK-12]: one test of the header */
#elif VARIANT == W_PLAINDATA
    return AT_LOAD32(&p->h.count) != 0 ? p : NULL;                      /* salvage: one atomic load and test */
#else
    uint32_t c = AT_LOAD32(&p->h.count);                                /* [WK-12] [RT-10]: compare-exchange loop */
    for (;;) {
        if (c == 0) return NULL;                                        /* ended: never raised from zero (ODR-060) */
        if (UNLIKELY(c >= 0x7FFFFFFFu))                                 /* [RC-4]: the thread_shared limit */
            e_stop("too many owners of a `Player` (2,147,483,647)");
        uint32_t seen = AT_CAS32(&p->h.count, c, c + 1);
        if (seen == c) return p;                                        /* [RC-5] point 2: the view holds one count */
        c = seen;
    }
#endif
}
static inline void view_end(const Player *p) {
#if VARIANT == W_TS
    owner_sub((Player *)p);                                             /* [RC-5] point 2: the view's count given back */
#else
    (void)p;                                                            /* R10 point 1 / salvage: nothing */
#endif
}
static inline void replace_player(uint32_t j, int64_t id) {            /* players[j] = thread_shared Player(...) */
    Player *n = new_player(j, id);
    Player *old = players[j];
    players[j] = n;
    owner_end(old);
}
#endif

int main(void) {
    h_start();
    world_init();
    Link *cams = (Link *)malloc(WC * sizeof(Link));
    if (!cams) abort();
    for (uint32_t i = 0; i < WC; i++) link_set(&cams[i], (i * WMULT) & (WP - 1));
    uint64_t acc = 0, fails = 0;
    for (uint32_t f = 0; f < WF; f++) {
        uint32_t r = f & 63;
        for (uint32_t j = r; j < WP; j += 64) replace_player(j, (int64_t)WP * (f + 1) + j);
        for (uint32_t i = 0; i < WC; i++) {
            const Player *p = link_open(&cams[i]);
            if (p) {
                acc += (uint64_t)p->x;
                view_end(p);
            } else {
                link_retarget(&cams[i], (i * WMULT) & (WP - 1));
                fails += 1;
            }
        }
    }
    h_report(NAME, acc + fails);
    return 0;
}
