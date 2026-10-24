/* share.c: the shared-object workload of the ownership group (A9, A10, A24, A34).
 *
 * 262,144 Player objects are shared by 1,048,576 watchers (cameras, HUDs, turrets): each watcher holds a
 * handle to one player. Every frame, every watcher changes its player's `hp` (a plain field), reads `x`
 * and the length of `inventory` (a list field); one watcher in eight appends to the inventory and, past
 * eight items, sums and clears it; one in sixteen re-targets to a random player (a handle store).
 *
 * One file, eight models, chosen with -DMODEL=n (same work, same random numbers, same checksum):
 *   0 plain      plain C: a raw pointer, no header, no counts, no checks
 *   1 today      what today's Ember compiler emits for this program (read from `ember build --emit c`,
 *                target/redesign2/ownership/probes/a09_emit): a 24-byte header ([OBJ-1]), 8 bytes of
 *                access words ([EXC-19]), a retain and a release of `t = ws[i].target` every step; no
 *                access check (the compiler removed them for this program, [EXC-3])
 *   2 checked    today's layout, plus the per-access checks [EXC-19] specifies when [EXC-3] cannot remove
 *                them (a program with a held write anywhere, e.g. one `mut self` method on Player)
 *   3 proposed   the ownership group's proposal: an 8-byte header (count, type index), no access words,
 *                no checks (field effects prove them at compile time), and no count traffic for `t`
 *                (effects prove nothing in t's life writes Watcher.target or drops a Player); counts
 *                change only when a handle is stored (re-targeting)
 * Two more, to take the gap between 1 and 3 apart:
 *   4 today-layout   model 1's 64-byte object, without the per-step retain and release
 *   5 proposed+rc    model 3's 40-byte object, with model 1's per-step retain and release
 *   6 proposed-16    model 3 with a 16-byte header (strong, weak, type index, flags kept in the header)
 *   7 no-counts      model 3's 40-byte object with no count updates at all (not safe: only to separate
 *                    the cost of the header from the cost of counting when a handle is stored)
 * Integer overflow checks ([TYP-8]) are left out of every model and of plain C (not a memory-model cost).
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n share.c -o share_n.exe */
#include "../../../redesign/bench/harness.h"
/* share_chunk.c (second ownership attack): the lean class with NO header in the object. Objects of one
 * class come from 64 KiB chunks aligned to 64 KiB (a per-class slab); each chunk starts with an array of
 * the counts of its objects, and the objects (32 bytes, plain C's layout) follow. A handle is a plain
 * pointer to the object; its count is found from the address: chunk = address & ~0xFFFF, slot =
 * (address - chunk - 7168) / 32. The class of an object is its chunk's class (one class per chunk), so
 * no type index is needed in the object either. Same work, same random numbers, same checksum as share.c.
 *   -DMODEL=9   counts in the chunk (this file's model)
 *   -DMODEL=10  the same chunks and layout with no counts at all (plain C from the same chunks)
 * Build: clang -O2 -DMODEL=9 share_chunk.c -o share_chunk.exe */
#include <string.h>
#include <malloc.h>
#define PER_CHUNK_ALL 1792u

#ifndef MODEL
#define MODEL 0
#endif
#define P_COUNT (1u << 18)
#define W_COUNT (1u << 20)
#define FRAMES 32

typedef struct { int32_t *p; size_t len, cap; } IVec;   /* Ember's ember_vec and Rust's Vec: 24 bytes */

static __declspec(noinline) void ivec_grow(IVec *v) {
    size_t cap = v->cap ? v->cap * 2 : 4;                  /* [HEAP-2]: double from four */
    int32_t *p = (int32_t *)realloc(v->p, cap * sizeof(int32_t));
    if (!p) abort();
    v->p = p; v->cap = cap;
}
static inline void ivec_push(IVec *v, int32_t x) {
    if (v->len == v->cap) ivec_grow(v);
    v->p[v->len++] = x;
}

#if MODEL == 1 || MODEL == 2 || MODEL == 5
#define STEP_RC 1      /* a retain and a release of t every step */
#else
#define STEP_RC 0
#endif
#if MODEL == 4
#define TODAY_LAYOUT 1
#elif MODEL == 1 || MODEL == 2
#define TODAY_LAYOUT 1
#else
#define TODAY_LAYOUT 0
#endif

#if MODEL == 0 || MODEL == 9 || MODEL == 10
typedef struct Player { int32_t hp; int32_t x; IVec inv; } Player;                       /* 32 bytes */
#elif TODAY_LAYOUT
typedef struct { uint32_t strong, weak, access, flags; const void *ti; } ObjHeader;        /* [OBJ-1] */
typedef struct Player { ObjHeader h; uint32_t access_inv, access_pad; int32_t hp; int32_t x; IVec inv; } Player; /* 64 */
#elif MODEL == 6
typedef struct { uint32_t strong, weak, type_index, flags; } ObjHeader;                   /* 16 bytes */
typedef struct Player { ObjHeader h; int32_t hp; int32_t x; IVec inv; } Player;           /* 48 bytes */
#else
typedef struct { uint32_t strong, type_index; } ObjHeader;                                /* proposed */
typedef struct Player { ObjHeader h; int32_t hp; int32_t x; IVec inv; } Player;           /* 40 bytes */
#endif

typedef struct { Player *target; } Watcher;

static __declspec(noinline) void fail(const char *what) { fprintf(stderr, "panic: %s\n", what); exit(101); }

#if MODEL == 9
#define CHUNK 65536u
#define PER_CHUNK 1792u                       /* 1792 counts (7,168 bytes) + 1792 objects of 32 bytes = 64,512 */
#define OBJ_OFF (PER_CHUNK * 4u)
static inline uint32_t *count_of(Player *o) {
    uintptr_t a = (uintptr_t)o, chunk = a & ~(uintptr_t)(CHUNK - 1);
    return (uint32_t *)chunk + ((a - chunk - OBJ_OFF) >> 5);
}
static __declspec(noinline) void retain_slow(Player *o) { (void)o; fail("strong count overflow"); }
static __declspec(noinline) void release_slow(Player *o) { free(o->inv.p); /* the slot goes back to its chunk */ }
static inline void retain(Player *o) {
    uint32_t *c = count_of(o);
    if (*c - 1u < UINT32_MAX - 1u) { *c += 1; return; }
    retain_slow(o);
}
static inline void release(Player *o) {
    uint32_t *c = count_of(o);
    if (*c > 1) { *c -= 1; return; }
    release_slow(o);
}
#elif MODEL != 0 && MODEL != 10
/* The inline fast paths of ember_retain_plain / ember_release_plain (runtime/ember_rt/include/ember_rt.h). */
static __declspec(noinline) void retain_slow(Player *o) { (void)o; fail("strong count overflow"); }
static __declspec(noinline) void release_slow(Player *o) { free(o->inv.p); (void)o; /* back to the slab free list */ }
static inline void retain(Player *o) {
#if MODEL != 3 && MODEL != 6
    if (o != NULL && o->h.strong - 1u < UINT32_MAX - 1u) { o->h.strong += 1; return; }
#else
    if (o->h.strong - 1u < UINT32_MAX - 1u) { o->h.strong += 1; return; }   /* a class handle is never null */
#endif
    retain_slow(o);
}
static inline void release(Player *o) {
#if MODEL != 3 && MODEL != 6
    if (o != NULL && o->h.strong > 1) { o->h.strong -= 1; return; }
#else
    if (o->h.strong > 1) { o->h.strong -= 1; return; }
#endif
    release_slow(o);
}
#endif

#if MODEL == 2
/* [EXC-19]: bit 31 writer, bits 0..30 reader count. Not held: check only. Held: check and count. */
#define READ_CHECK(w)  do { if ((w) & 0x80000000u) fail("exclusivity: read while written"); } while (0)
#define WRITE_CHECK(w) do { if ((w) != 0) fail("exclusivity: write while accessed"); } while (0)
#else
#define READ_CHECK(w)  ((void)0)
#define WRITE_CHECK(w) ((void)0)
#endif

static char *chunk; static uint32_t chunk_used = PER_CHUNK_ALL;
static Player *new_player(uint32_t i) {
    if (chunk_used == PER_CHUNK_ALL) {
        chunk = (char *)_aligned_malloc(65536, 65536);
        if (!chunk) abort();
        memset(chunk, 0, 65536);
        chunk_used = 0;
    }
    Player *p = (Player *)(chunk + PER_CHUNK_ALL * 4u + (size_t)chunk_used * sizeof(Player));
#if MODEL == 9
    ((uint32_t *)chunk)[chunk_used] = 1;
#endif
    chunk_used++;
    memset(p, 0, sizeof *p);
#if TODAY_LAYOUT
    p->h.strong = 1; p->h.weak = 1; p->h.ti = (const void *)&new_player;
#elif MODEL != 0 && MODEL != 9 && MODEL != 10
    p->h.strong = 1; p->h.type_index = 7;
#endif
    p->hp = 1000; p->x = (int32_t)(i & 1023);
    return p;
}

int main(void) {
    h_start();
    h_seed(42);
    Player **players = (Player **)malloc(P_COUNT * sizeof(Player *));
    Watcher *ws = (Watcher *)malloc(W_COUNT * sizeof(Watcher));
    if (!players || !ws) abort();
    for (uint32_t i = 0; i < P_COUNT; i++) players[i] = new_player(i);
    for (uint32_t i = 0; i < W_COUNT; i++) {
        Player *t = players[h_rand() & (P_COUNT - 1)];
#if MODEL != 0 && MODEL != 7 && MODEL != 10
        retain(t);
#endif
        ws[i].target = t;
    }
    static volatile size_t vlen = P_COUNT;          /* the length as a run-time value, as players.len is */
    size_t players_len = vlen;
    (void)players_len;
    uint64_t acc = 0;
    for (uint32_t f = 0; f < FRAMES; f++) {
        for (uint32_t i = 0; i < W_COUNT; i++) {
            Player *t = ws[i].target;                           /* t = ws[i].target */
#if STEP_RC
            retain(t);                                          /* emitted today: t is a local handle */
#endif
            t->hp -= 1;                                         /* a plain field: never checked ([EXC-17]) */
            READ_CHECK(t->access_inv);                          /* len(t.inventory): a read, not held */
            acc += (uint64_t)(uint32_t)t->x * (uint64_t)(t->inv.len + 1);
            if ((i & 7) == (f & 7)) {
                WRITE_CHECK(t->access_inv);                     /* t.inventory.push(f): a write, not held */
                ivec_push(&t->inv, (int32_t)f);
                READ_CHECK(t->access_inv);
                if (t->inv.len > 8) {
#if MODEL == 2
                    READ_CHECK(t->access_inv); t->access_inv += 1;   /* for v in t.inventory: a held read */
#endif
                    for (size_t k = 0; k < t->inv.len; k++) acc += (uint64_t)(uint32_t)t->inv.p[k];
#if MODEL == 2
                    t->access_inv -= 1;
#endif
                    WRITE_CHECK(t->access_inv);                 /* t.inventory.clear() */
                    t->inv.len = 0;
                }
            }
#if STEP_RC
            release(t);
#endif
            if ((h_rand() & 15) == 0) {                          /* ws[i].target = players[r] */
                uint32_t r = h_rand() & (P_COUNT - 1);
#if MODEL == 7 || MODEL == 10
                if (r >= players_len) fail("index out of bounds");
                ws[i].target = players[r];
#elif MODEL != 0
                if (r >= players_len) fail("index out of bounds");  /* Ember checks players[r] */
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
    h_report("share-plain-chunk", sum);
#elif MODEL == 1
    h_report("share-today", sum);
#elif MODEL == 2
    h_report("share-today-checked", sum);
#elif MODEL == 3
    h_report("share-proposed-slab", sum);
#elif MODEL == 9
    h_report("share-lean-counts-in-chunk", sum);
#elif MODEL == 10
    h_report("share-chunk-no-counts", sum);
#elif MODEL == 4
    h_report("share-today-layout-only", sum);
#elif MODEL == 6
    h_report("share-proposed-16b-header", sum);
#elif MODEL == 7
    h_report("share-proposed-no-counts", sum);
#else
    h_report("share-proposed-plus-step-rc", sum);
#endif
    return 0;
}
