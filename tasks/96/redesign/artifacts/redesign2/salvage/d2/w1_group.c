/* SALVAGED "checked pointers" (phase3-design-D2.md), W1 in EnTT's owning-group layout.
   What changed from target/redesign/bench/D2/w1.c, and why:
   - FIX (layout, measured cause of D2's W1 cost): the occupant of a pool slot is EnTT's own packed
     entity array, ents[k]; it IS the slot's generation. D2's model put a 64-bit owner word beside
     every component (16-byte slots, tombstones, no swap-and-pop); here a component reference is
     {pool, slot k, entity e}, valid exactly when k < pool.n and ents[k] == e. Swap-and-pop is
     allowed again, because a reference re-resolves by index and checks the occupant, so slots no
     longer need to stay put and pools are ordinary growable arrays (no reserved address ranges).
   - Entity ids are 32-bit serials, never reused (D2's rule, at EnTT's width). A design that recycles
     ids gives each index a version and retires the index when its version runs out ([HND-3]).
   Kept from D2: the registry's validity bitset checked by destroy; destroy calls each pool's remove
   through its interface table; P5 (references produced by the iteration step and used before any
   change need no check: no occupant check in the loop); L5 (a loop whose body may change the pools
   re-reads their buffers and lengths each step, with bounds checks).
   Left out, as in every model: [TYP-8] overflow checks. */
#include "../sh.h"
#include <string.h>

#define N 1000000u
#define NONE 0xFFFFFFFFu
static void d2_panic(const char *m) { fprintf(stderr, "panic: %s\n", m); fflush(stderr); ExitProcess(101); }
static void *xrealloc(void *p, size_t n) { void *q = realloc(p, n); if (!q) d2_panic("out of memory"); return q; }

typedef struct { float x, y; } V2; /* Position and Velocity have the same layout */
typedef struct Pool Pool;
typedef struct { void (*remove)(Pool *, uint32_t); } AnyPoolVT;
struct Pool {
    const AnyPoolVT *vt;
    uint32_t *sparse; size_t scap;      /* entity id -> slot */
    uint32_t *ents; V2 *data; size_t n, cap; /* ents[k]: the occupant of slot k (its generation) */
};
typedef struct {
    uint64_t *alive; size_t acap;       /* bit per entity id */
    uint32_t next;
    uint64_t alive_count;
    Pool *pools[2];                     /* by compile-time component index */
    size_t glen;                        /* the owning group <Position, Velocity> */
} Registry;

static void sparse_cover(Pool *p, uint32_t id) {
    if (id < p->scap) return;
    size_t nc = p->scap ? p->scap : 1024;
    while (nc <= id) nc *= 2;
    p->sparse = xrealloc(p->sparse, nc * 4);
    for (size_t i = p->scap; i < nc; i++) p->sparse[i] = NONE;
    p->scap = nc;
}
static inline int pool_has(const Pool *p, uint32_t id) { return id < p->scap && p->sparse[id] != NONE; }
static void pool_add(Pool *p, uint32_t id, V2 v) {
    sparse_cover(p, id);
    if (p->sparse[id] != NONE) d2_panic("emplace: the entity already has this component");
    if (p->n == p->cap) {
        p->cap = p->cap ? p->cap * 2 : 4;
        p->ents = xrealloc(p->ents, p->cap * 4); p->data = xrealloc(p->data, p->cap * sizeof(V2));
    }
    p->sparse[id] = (uint32_t)p->n; p->ents[p->n] = id; p->data[p->n] = v; p->n++;
}
static void pool_swap(Pool *p, size_t i, size_t j) {
    if (i == j) return;
    if (i >= p->n || j >= p->n) d2_panic("index out of range");
    uint32_t a = p->ents[i], b = p->ents[j]; V2 t = p->data[i];
    p->ents[i] = b; p->ents[j] = a; p->data[i] = p->data[j]; p->data[j] = t;
    p->sparse[b] = (uint32_t)i; p->sparse[a] = (uint32_t)j;
}
static void pool_remove(Pool *p, uint32_t id) { /* swap-and-pop; the moved slot's occupant moves with it */
    if (!pool_has(p, id)) return;
    uint32_t i = p->sparse[id], last = p->ents[--p->n];
    p->ents[i] = last; p->data[i] = p->data[p->n]; p->sparse[last] = i; p->sparse[id] = NONE;
}
static const AnyPoolVT pos_vt = {pool_remove}, vel_vt = {pool_remove};

static uint32_t reg_create(Registry *r) {
    uint32_t e = r->next++;
    if (r->next == 0) d2_panic("entity ids exhausted");
    if ((e >> 6) >= r->acap) {
        size_t nc = r->acap ? r->acap * 2 : 1024;
        r->alive = xrealloc(r->alive, nc * 8);
        memset(r->alive + r->acap, 0, (nc - r->acap) * 8);
        r->acap = nc;
    }
    r->alive[e >> 6] |= 1ull << (e & 63);
    r->alive_count++;
    return e;
}
static int reg_valid(const Registry *r, uint32_t e) { return e < r->next && ((r->alive[e >> 6] >> (e & 63)) & 1); }
static void group_maybe_enter(Registry *r, uint32_t e) {
    Pool *a = r->pools[0], *b = r->pools[1];
    if (!pool_has(a, e) || !pool_has(b, e)) return;
    pool_swap(a, a->sparse[e], r->glen); pool_swap(b, b->sparse[e], r->glen); r->glen++;
}
static void reg_emplace(Registry *r, int ci, uint32_t e, V2 v) {
    if (!reg_valid(r, e)) d2_panic("emplace: the entity is not alive");
    pool_add(r->pools[ci], e, v);
    group_maybe_enter(r, e);
}
static void reg_destroy(Registry *r, uint32_t e) {
    if (!reg_valid(r, e)) d2_panic("destroy: the entity is not alive");
    Pool *a = r->pools[0], *b = r->pools[1];
    if (pool_has(a, e) && pool_has(b, e) && a->sparse[e] < r->glen) {
        r->glen--; pool_swap(a, a->sparse[e], r->glen); pool_swap(b, b->sparse[e], r->glen);
    }
    for (int i = 0; i < 2; i++) r->pools[i]->vt->remove(r->pools[i], e);
    r->alive[e >> 6] &= ~(1ull << (e & 63));
    r->alive_count--;
}

int main(void) {
    h_start();
    static Pool pos = {&pos_vt}, vel = {&vel_vt};
    static Registry reg;
    reg.pools[0] = &pos; reg.pools[1] = &vel;
    for (uint32_t i = 0; i < N; i++) {
        uint32_t e = reg_create(&reg);
        reg_emplace(&reg, 0, e, (V2){(float)(i % 1000), (float)(i / 1000)});
        if (i % 2 == 0) reg_emplace(&reg, 1, e, (V2){(float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f});
    }
    H_PHASE("setup");
    for (uint32_t s = 0; s < 100; s++) {
        /* for e, p, v in reg.group[(Position, Velocity)](): k = glen-1 .. 0 (L5: re-read each step) */
        for (size_t k = reg.glen; k-- > 0;) {
            if (k >= pos.n || k >= vel.n) d2_panic("index out of range");
            uint32_t e = pos.ents[k];
            V2 *p = &pos.data[k], *v = &vel.data[k]; /* P5: fresh, used before any change: no check */
            p->x += v->x;
            p->y += v->y;
            if (e % 100 == s) {
                reg_destroy(&reg, e);
                uint32_t n = reg_create(&reg);
                reg_emplace(&reg, 0, n, (V2){0.0f, 0.0f});
                reg_emplace(&reg, 1, n, (V2){1.0f, 1.0f});
            }
        }
    }
    H_PHASE("steps");
    uint64_t sum = 0;
    for (size_t j = 0; j < pos.n; j++)
        sum += (uint64_t)((int64_t)(pos.data[j].x * 4.0f) + (int64_t)(pos.data[j].y * 4.0f));
    h_report("d2-w1-group", sum + reg.alive_count);
    return 0;
}
