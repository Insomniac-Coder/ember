/* share_weak.c: the ownership attack's sharing workload (ownership-attack2/bench/share_slab.c, the
   lean class with objects from one block), with the combined model's count word, when the players
   are ALSO the targets of `weak` fields (a camera, a HUD, an observer list). Problem 5 of
   tasks/96/redesign/phase3b-combined-attack.md: a cost that appears only when counted class handles
   (ownership group) meet the combined model's `weak` (counts move to a side record).

   The work is share_slab.c's, line for line: 262,144 players, 1,048,576 watchers holding handles,
   32 frames; every step changes hp, reads x and the inventory length, one step in eight pushes (and
   past eight items sums and clears), one in sixteen re-targets (a retain and a release).
   Models (-DMODEL=n), same random numbers, same checksum:
     0  plain C, no header, no counts (share_slab0.c)
     3  the combined model's count word, no weak referrers: the fast path is one subtract and compare
     8  the combined model as written, every player weakly referenced: the count word holds SIDE and a
        side-record index, so every retain and release of a player goes to the side record
     9  the fix (problems 2 and 5): the strong count never leaves the header; the weak counts live
        beside the objects, one 32-bit count per slab slot in an array the slab chunk points to (here
        the players' one block is one chunk); the count word and the fast paths are model 3's
   Build: clang -O2 -DMODEL=n share_weak.c -o share_weak_n.exe */
#include "../../redesign/bench/harness.h"
#include <string.h>
#ifndef MODEL
#define MODEL 0
#endif
#define P_COUNT (1u << 18)
#define W_COUNT (1u << 20)
#define FRAMES 32

typedef struct { int32_t *p; size_t len, cap; } IVec;
static __declspec(noinline) void ivec_grow(IVec *v) {
    size_t cap = v->cap ? v->cap * 2 : 4;
    int32_t *p = (int32_t *)realloc(v->p, cap * sizeof(int32_t));
    if (!p) abort();
    v->p = p; v->cap = cap;
}
static inline void ivec_push(IVec *v, int32_t x) { if (v->len == v->cap) ivec_grow(v); v->p[v->len++] = x; }

#if MODEL == 0
typedef struct Player { int32_t hp; int32_t x; IVec inv; } Player;
#else
typedef struct Player { uint32_t rc, type_index; int32_t hp; int32_t x; IVec inv; } Player;   /* 40 bytes */
#endif
typedef struct { Player *target; } Watcher;
static __declspec(noinline) void fail(const char *what) { fprintf(stderr, "panic: %s\n", what); exit(101); }

#define SHARED (1u << 31)
#define CLAIM  (1u << 30)
#define SIDE   (1u << 29)
#define DEAD   (1u << 28)
#define LOW    (DEAD - 1u)



#if MODEL == 8
typedef struct { uint64_t strong, weak; } SideRec;
static SideRec *side; static uint32_t side_len;
#endif

#if MODEL == 3 || MODEL == 8 || MODEL == 9
static __declspec(noinline) void retain_slow(Player *o) {
#if MODEL == 8
    if (o->rc & SIDE) { side[o->rc & LOW].strong++; return; }
#endif
    fail("count past 28 bits (not reached)");
}
static __declspec(noinline) void release_slow(Player *o) {
#if MODEL == 8
    if (o->rc & SIDE) { if (--side[o->rc & LOW].strong == 0) fail("a player died (not reached)"); return; }
#endif
    fail("a player died (not reached)");
}
static inline void retain(Player *o) {
    uint32_t rc = o->rc;
    if (__builtin_expect(rc - 1u < LOW - 1u, 1)) { o->rc = rc + 1; return; }   /* 1..2^28-2, no flag */
    retain_slow(o);
}
static inline void release(Player *o) {
    uint32_t rc = o->rc;
    if (__builtin_expect(rc - 2u < LOW - 2u, 1)) { o->rc = rc - 1; return; }   /* 2..2^28-1, no flag */
    release_slow(o);
}
#endif

static Player *slab; static uint32_t slab_used;
static Player *new_player(uint32_t i) {
    if (!slab) { slab = (Player *)malloc((size_t)P_COUNT * sizeof(Player)); if (!slab) abort(); }
    Player *p = &slab[slab_used++];
    memset(p, 0, sizeof *p);
#if MODEL != 0
    p->rc = 1; p->type_index = 7;
#endif
    p->hp = 1000; p->x = (int32_t)(i & 1023);
    return p;
}
/* one weak referrer per player (a camera or an observer list holds `weak Player`), set up before the
   timed loop as a program would: model 8 moves the count into a side record (combined/w3.c's
   weak_retain); model 9 adds to the weak count beside the object (the count word is untouched) */
static void add_weak_referrer(Player *p) {
#if MODEL == 8
    if (!side) { side = (SideRec *)malloc((size_t)P_COUNT * sizeof(SideRec)); if (!side) abort(); }
    uint32_t i = side_len++;
    side[i].strong = p->rc & LOW; side[i].weak = 1;
    p->rc = (p->rc & ~LOW) | SIDE | i;
#elif MODEL == 9
    /* the chunk's array of weak counts, one per slot (allocated when the first object in the chunk
       becomes a weak target); 2 = one weak field plus the strong handles' extra one */
    static uint32_t *weak_beside;
    if (!weak_beside) { weak_beside = calloc(P_COUNT, sizeof(uint32_t)); if (!weak_beside) abort(); }
    uint32_t *w = &weak_beside[p - slab];
    *w = *w ? *w + 1 : 2;
#else
    (void)p;
#endif
}

int main(void) {
    h_start();
    h_seed(42);
    Player **players = (Player **)malloc(P_COUNT * sizeof(Player *));
    Watcher *ws = (Watcher *)malloc(W_COUNT * sizeof(Watcher));
    if (!players || !ws) abort();
    for (uint32_t i = 0; i < P_COUNT; i++) { players[i] = new_player(i); add_weak_referrer(players[i]); }
    for (uint32_t i = 0; i < W_COUNT; i++) {
        Player *t = players[h_rand() & (P_COUNT - 1)];
#if MODEL != 0
        retain(t);
#endif
        ws[i].target = t;
    }
    static volatile size_t vlen = P_COUNT;
    size_t players_len = vlen;
    (void)players_len;
    uint64_t acc = 0;
    for (uint32_t f = 0; f < FRAMES; f++) {
        for (uint32_t i = 0; i < W_COUNT; i++) {
            Player *t = ws[i].target;
            t->hp -= 1;
            acc += (uint64_t)(uint32_t)t->x * (uint64_t)(t->inv.len + 1);
            if ((i & 7) == (f & 7)) {
                ivec_push(&t->inv, (int32_t)f);
                if (t->inv.len > 8) {
                    for (size_t k = 0; k < t->inv.len; k++) acc += (uint64_t)(uint32_t)t->inv.p[k];
                    t->inv.len = 0;
                }
            }
            if ((h_rand() & 15) == 0) {
                uint32_t r = h_rand() & (P_COUNT - 1);
#if MODEL != 0
                if (r >= players_len) fail("index out of bounds");
                Player *n = players[r];
                retain(n);
                Player *old = ws[i].target;
                ws[i].target = n;
                release(old);
#else
                ws[i].target = players[r];
#endif
            }
        }
    }
    uint64_t sum = acc;
    for (uint32_t i = 0; i < P_COUNT; i++) sum += (uint64_t)(uint32_t)players[i]->hp + players[i]->inv.len;
#if MODEL == 0
    h_report("share-plain-slab", sum);
#elif MODEL == 3
    h_report("share-lean-no-weak", sum);
#elif MODEL == 8
    h_report("share-lean-weak-side-record", sum);
#else
    h_report("share-lean-weak-count-beside", sum);
#endif
    return 0;
}
