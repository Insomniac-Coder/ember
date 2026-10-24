/* SALVAGED "today's rules, but never an error" (phase3-design-D3.md), W1 in EnTT's owning-group layout.
   What changed from target/redesign/bench/D3/w1.c, and why:
   - FIX (the PIN is not needed): the view's cursor is an INDEX that the library re-resolves at each
     step (bounds-checked), not a pointer held across the body, and the references it yields
     (`pos`, `vel`) end at their last use, before `reg.destroy(e)` (non-lexical lifetimes, which the
     static checker D3 keeps already has). So no borrow is live across the change and the borrow
     checker proves the loop with no fallback: no pin counter, no tombstones, no overflow pages, no
     settle. The despawn swap-and-pops at once (EnTT's order), exactly as in plain C.
   - the layout: EnTT's owning group <Position, Velocity>.
   Kept from D3: pools are objects in the registry's table, indexed by the program's component type
   number (a compile-time constant) with a 4-byte type index; destroy reaches each pool through the
   type-erased table; entity ids from a counter, never reused; bounds checks the compiler cannot
   prove (the cursor against both pools, re-read after the body may have changed them).
   Left out, as in every model: [TYP-8] overflow checks. */
#include "../sh.h"
#include <string.h>

#define N 1000000u
#define NIL 0xFFFFFFFFu
static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)
static void *xrealloc(void *p, size_t n) { void *q = realloc(p, n); CHECK(q, "out of memory"); return q; }

typedef struct { float x, y; } V2;
enum { T_Position = 0, T_Velocity = 1, NTYPES = 2 };
typedef struct { uint32_t *data; int64_t len, cap; } ListU32;
typedef struct {
    uint32_t type;                      /* object header: type index (interface dispatch) */
    V2 *dense; uint32_t *ents; int64_t len, cap;
    ListU32 sparse;
} Pool;
typedef struct { Pool *pools[NTYPES]; uint32_t next_id; int64_t glen; } Registry;

static void sparse_cover(ListU32 *s, uint32_t e) {
    if ((int64_t)e < s->len) return;
    int64_t need = (int64_t)e + 1;
    if (need > s->cap) {
        int64_t c = s->cap ? s->cap : 4;
        while (c < need) c *= 2;
        s->data = xrealloc(s->data, (size_t)c * 4);
        s->cap = c;
    }
    for (int64_t i = s->len; i < need; i++) s->data[i] = NIL;
    s->len = need;
}
static inline int pool_has(const Pool *p, uint32_t e) { return (int64_t)e < p->sparse.len && p->sparse.data[e] != NIL; }
static void pool_add(Pool *p, uint32_t e, V2 v) {
    sparse_cover(&p->sparse, e);
    CHECK(p->sparse.data[e] == NIL, "entity already has this component");
    if (p->len == p->cap) {
        int64_t c = p->cap ? p->cap * 2 : 4;
        p->dense = xrealloc(p->dense, (size_t)c * sizeof(V2)); p->ents = xrealloc(p->ents, (size_t)c * 4);
        p->cap = c;
    }
    p->sparse.data[e] = (uint32_t)p->len; p->ents[p->len] = e; p->dense[p->len] = v; p->len++;
}
static void pool_swap(Pool *p, int64_t i, int64_t j) {
    if (i == j) return;
    CHECK(i >= 0 && i < p->len && j >= 0 && j < p->len, "index out of bounds");
    uint32_t a = p->ents[i], b = p->ents[j]; V2 t = p->dense[i];
    p->ents[i] = b; p->ents[j] = a; p->dense[i] = p->dense[j]; p->dense[j] = t;
    p->sparse.data[b] = (uint32_t)i; p->sparse.data[a] = (uint32_t)j;
}
static void pool_remove(void *pv, uint32_t e) { /* swap-and-pop, as EnTT */
    Pool *p = pv;
    if (!pool_has(p, e)) return;
    uint32_t j = p->sparse.data[e];
    p->sparse.data[e] = NIL;
    int64_t last = p->len - 1;
    if ((int64_t)j != last) { p->dense[j] = p->dense[last]; p->ents[j] = p->ents[last]; p->sparse.data[p->ents[j]] = j; }
    p->len--;
}
typedef struct { void (*remove)(void *, uint32_t); } AnyPoolVT;
static const AnyPoolVT ANYPOOL[NTYPES] = {{pool_remove}, {pool_remove}};

static Pool *reg_pool(Registry *r, int t) {
    if (!r->pools[t]) { r->pools[t] = calloc(1, sizeof(Pool)); CHECK(r->pools[t], "out of memory"); r->pools[t]->type = (uint32_t)t; }
    return r->pools[t];
}
static uint32_t reg_create(Registry *r) { CHECK(r->next_id != NIL, "entity ids exhausted"); return r->next_id++; }
static void group_maybe_enter(Registry *r, uint32_t e) {
    Pool *a = reg_pool(r, T_Position), *b = reg_pool(r, T_Velocity);
    if (!pool_has(a, e) || !pool_has(b, e)) return;
    pool_swap(a, a->sparse.data[e], r->glen); pool_swap(b, b->sparse.data[e], r->glen); r->glen++;
}
static void reg_emplace(Registry *r, int t, uint32_t e, V2 v) { pool_add(reg_pool(r, t), e, v); group_maybe_enter(r, e); }
static void reg_destroy(Registry *r, uint32_t e) {
    Pool *a = r->pools[T_Position], *b = r->pools[T_Velocity];
    if (a && b && pool_has(a, e) && pool_has(b, e) && a->sparse.data[e] < r->glen) {
        r->glen--; pool_swap(a, a->sparse.data[e], r->glen); pool_swap(b, b->sparse.data[e], r->glen);
    }
    for (int t = 0; t < NTYPES; t++) {
        Pool *p = r->pools[t];
        if (p) ANYPOOL[p->type].remove(p, e);
    }
}

int main(void) {
    h_start();
    Registry *r = calloc(1, sizeof *r);
    CHECK(r, "out of memory");
    for (uint32_t i = 0; i < N; i++) {
        uint32_t e = reg_create(r);
        reg_emplace(r, T_Position, e, (V2){(float)(i % 1000), (float)(i / 1000)});
        if (i % 2 == 0) reg_emplace(r, T_Velocity, e, (V2){(float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f});
    }
    H_PHASE("setup");
    for (uint32_t s = 0; s < 100; s++) {
        /* for e, pos, vel in reg.group[(Position, Velocity)](): the cursor is an index */
        Pool *pp = r->pools[T_Position], *vp = r->pools[T_Velocity];
        for (int64_t k = r->glen - 1; k >= 0; k--) {
#ifndef NOBOUNDS /* attribution build: -DNOBOUNDS drops the cursor's two bounds checks */
            CHECK(k < pp->len && k < vp->len, "index out of bounds");
#endif
            uint32_t e = pp->ents[k];
            V2 *pos = &pp->dense[k], *vel = &vp->dense[k];   /* both borrows end below */
            pos->x += vel->x;
            pos->y += vel->y;
            if (e % 100u == s) {
                reg_destroy(r, e);
                uint32_t n = reg_create(r);
                reg_emplace(r, T_Position, n, (V2){0.0f, 0.0f});
                reg_emplace(r, T_Velocity, n, (V2){1.0f, 1.0f});
            }
        }
    }
    H_PHASE("steps");
    Pool *pp = r->pools[T_Position];
    uint64_t sum = 0;
    for (int64_t i = 0; i < pp->len; i++)
        sum += (uint64_t)((int64_t)(pp->dense[i].x * 4.0f) + (int64_t)(pp->dense[i].y * 4.0f));
    h_report("d3-w1-group", sum + (uint64_t)pp->len);
    return 0;
}
