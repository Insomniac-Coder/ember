/* SALVAGED "checked pointers" (phase3-design-D2.md), W3: game objects that refer to each other.
   Built from target/redesign/bench/D2/w3.c (one store for the open hierarchy, 40-byte slots that never
   move with {generation, class id}, the virtual update through a class-id table, the target check as
   the program's own question, destroy by generation bump, LIFO slot reuse, slot retirement). Fixes:
   - FIX 1 (the program, breaker's +8%): the removal is written `objects.retain(fn(o) => o.health > 0)`
     (one compaction pass), not `objects.remove(o)` inside the loop, which is Python's linear search.
   - FIX 2 (the reference): one 64-bit word {48-bit address, 16-bit generation}; following it is one
     AND instead of base + index * 40. None is a reference to a reserved dead slot, so `t is not None`
     and "t was destroyed" are the same one compare, as in D2.
   Left out, as in every model: [TYP-8] overflow checks. */
#include "../sh.h"
#include <string.h>

#define M 100000u
#define FRAMES 1000u
static void d2_panic(const char *m) { fprintf(stderr, "panic: %s\n", m); fflush(stderr); ExitProcess(101); }

typedef struct { char *base; size_t reserved, committed; } Range;
static void range_init(Range *r, size_t reserve) {
    r->base = (char *)VirtualAlloc(NULL, reserve, MEM_RESERVE, PAGE_NOACCESS);
    if (!r->base) d2_panic("address space reservation failed");
    r->reserved = reserve; r->committed = 0;
}
static void range_ensure(Range *r, size_t bytes) {
    if (bytes <= r->committed) return;
    size_t chunk = (size_t)1 << 20, want = (bytes + chunk - 1) & ~(chunk - 1);
    if (want > r->reserved) d2_panic("store capacity exhausted");
    if (!VirtualAlloc(r->base + r->committed, want - r->committed, MEM_COMMIT, PAGE_READWRITE)) d2_panic("out of memory");
    r->committed = want;
}

typedef uint64_t GRef;
#define GADDR(r) ((r) & 0x0000FFFFFFFFFFFFull)
#define GGEN(r) ((uint32_t)((r) >> 48))
typedef struct ObjSlot {
    uint32_t gen;     /* odd = live */
    uint32_t cls;     /* 0 Ship, 1 Rock, 2 Turret; the free-list link while dead */
    int64_t id;
    int32_t x, y, health;
    uint32_t pad;
    GRef target;      /* Option[Object] */
} ObjSlot;            /* 40 bytes */
_Static_assert(sizeof(ObjSlot) == 40, "slot layout");
#define OBJ(r) ((ObjSlot *)GADDR(r))
#define NO_SLOT UINT32_MAX

static Range store; static uint32_t store_len, free_top = NO_SLOT;
static GRef NONE_REF;  /* slot 0: reserved, dead */
static inline ObjSlot *slot(uint32_t s) { return (ObjSlot *)store.base + s; }
static inline GRef gref(ObjSlot *o) { return (GRef)(uintptr_t)o | ((GRef)(o->gen & 0xFFFF) << 48); }
static inline int is_live(GRef r) { return (OBJ(r)->gen & 0xFFFF) == GGEN(r); }

static GRef obj_alloc(void) {
    uint32_t s = free_top;
    if (s != NO_SLOT) free_top = slot(s)->cls;
    else { s = store_len++; range_ensure(&store, (size_t)store_len * sizeof(ObjSlot)); }
    ObjSlot *o = slot(s);
    o->gen += 1; /* odd */
    return gref(o);
}
static void obj_destroy(GRef r) {
    ObjSlot *o = OBJ(r);
    if (!is_live(r)) d2_panic("destroy: stale reference"); /* breaker H7: destroy checks the generation */
    o->gen += 1;                                            /* even: every reference to it is stale */
    if ((o->gen & 0xFFFF) == 0xFFFE) return;                /* 16-bit generation exhausted: retired */
    o->cls = free_top;
    free_top = (uint32_t)(o - (ObjSlot *)store.base);
}

typedef struct { GRef *ptr; int64_t len, cap; } RefList;
static void reflist_push(RefList *l, GRef r) {
    if (l->len == l->cap) {
        int64_t nc = l->cap ? l->cap * 2 : 4;
        GRef *np = realloc(l->ptr, (size_t)nc * sizeof(GRef));
        if (!np) d2_panic("out of memory");
        l->ptr = np; l->cap = nc;
    }
    l->ptr[l->len++] = r;
}

static inline int32_t sign(int32_t d) { return (d > 0) - (d < 0); }
static void ship_update(ObjSlot *o) {            /* self: live (P1/P6) */
    GRef t = o->target;
    if (is_live(t)) { ObjSlot *to = OBJ(t); o->x += sign(to->x - o->x); o->y += sign(to->y - o->y); }
}
static void rock_update(ObjSlot *o) { o->x = (o->x + 1) % 1000; }
static void turret_update(ObjSlot *o) {
    GRef t = o->target;
    if (is_live(t)) { ObjSlot *to = OBJ(t); if (abs(to->x - o->x) + abs(to->y - o->y) < 300) to->health -= 1; }
}
static void (*const UPDATE[3])(ObjSlot *) = {ship_update, rock_update, turret_update};

static GRef object_new(int64_t id, int32_t x, int32_t y, GRef target) {
    GRef r = obj_alloc();
    ObjSlot *o = OBJ(r);
    o->cls = (uint32_t)(id % 3); o->id = id; o->x = x; o->y = y; o->health = (int32_t)(100 + id % 50);
    o->target = target;
    return r;
}

int main(void) {
    h_start();
    h_seed(7);
    range_init(&store, (size_t)256 << 20);
    range_ensure(&store, sizeof(ObjSlot));
    slot(0)->gen = 2; store_len = 1;
    NONE_REF = gref(slot(0)) & 0x0000FFFFFFFFFFFFull; /* generation 0 never matches the slot's 2 */

    RefList objects = {0};
    for (uint32_t i = 0; i < M; i++) {
        int32_t x = (int32_t)(h_rand() % 1000), y = (int32_t)(h_rand() % 1000);
        GRef t = NONE_REF;
        if (i != 0) {
            uint32_t k = h_rand() % i;
            if ((int64_t)k >= objects.len) d2_panic("index out of range");
            t = objects.ptr[k];
        }
        reflist_push(&objects, object_new(i, x, y, t));
    }
    int64_t next_id = M;
    uint64_t destroyed_total = 0;
    for (uint32_t f = 0; f < FRAMES; f++) {
        for (int64_t i = 0; i < objects.len; i++) { ObjSlot *o = OBJ(objects.ptr[i]); UPDATE[o->cls](o); }
        /* dead = objects.retain(fn(o) => o.health > 0): destroyed in increasing id order */
        int64_t w = 0, before = objects.len;
        for (int64_t i = 0; i < before; i++) {
            GRef r = objects.ptr[i];
            if (OBJ(r)->health > 0) objects.ptr[w++] = r; else obj_destroy(r);
        }
        objects.len = w;
        int64_t dead = before - w;
        destroyed_total += (uint64_t)dead;
        for (int64_t k = 0; k < dead; k++) {
            int64_t id = next_id++;
            int32_t x = (int32_t)(h_rand() % 1000), y = (int32_t)(h_rand() % 1000);
            GRef t = NONE_REF;
            if (objects.len > 0) t = objects.ptr[h_rand() % (uint32_t)objects.len];
            reflist_push(&objects, object_new(id, x, y, t));
        }
    }
    int64_t s = 0;
    for (int64_t i = 0; i < objects.len; i++) { ObjSlot *o = OBJ(objects.ptr[i]); s += (int64_t)o->x + o->y + o->health; }
    h_report("d2-w3", (uint64_t)s + 1000000ull * destroyed_total);
    return 0;
}
