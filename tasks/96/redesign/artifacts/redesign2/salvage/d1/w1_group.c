/* SALVAGED "copy instead of share" (phase3-design-D1.md), W1 in EnTT's owning-group layout.
   Built from target/redesign/bench/D1/w1.c: the same runtime (d1rt.h), pools as counted
   `dyn AnyPool` boxes in an Array, a downcast compare and a box uniqueness test per pool projection,
   `remove` through the type table, shift versions. Changed:
   - the layout: EnTT's owning group <Position, Velocity> (the group length lives in the registry
     and is kept by emplace and destroy, as EnTT's group handler does);
   - FIX for breaker M1: the group generator is inlined into the user's loop (a required compile
     step: "a generator consumed by a `for` whose body is visible is compiled into that loop", the
     transformation Rust relies on for iterators). Built with -DNOINLINE, every resume instead fetches
     both pool projections again (what D1-6 demands of an out-of-line generator): that build measures
     what the fix removes.
   Safety fix included (breaker B6, the callee rule): `Registry.destroy(mut self)` fetches `self.pools`
   again after each `dyn` remove call (856,653 destroys, two calls each).
   Checks kept: uniqueness tests (hoisted, repeated after the destroy branch), range checks of the
   cursor against the three buffers (hoisted, repeated after the destroy branch), every check inside
   the library's emplace / remove / group code. Left out, as in every model: [TYP-8] overflow checks. */
#include "d1rt.h"

typedef struct { float x, y; } Position;
typedef struct { float dx, dy; } Velocity;
typedef struct { float a, b; } Comp8; /* Pool[Position] and Pool[Velocity] compile to the same code */
#define NONE 0xFFFFFFFFu

typedef struct Pool { Buf *sparse; Buf *packed; Buf *dense; } Pool;
typedef struct PoolVT { uint32_t type_index; void (*remove)(Pool *, uint32_t); } PoolVT;
typedef struct DynBox { uint32_t count; uint32_t pad_; Pool payload; } DynBox;
typedef struct Dyn { const PoolVT *vt; DynBox *box; } Dyn;
typedef struct Registry { Buf *pools; uint32_t next; uint64_t glen; } Registry;

static void buf_retain1(Buf *b) { if (b != &EMPTY) b->count++; }
static void dyn_retain(void *el, uint64_t n) {
    Dyn *d = el;
    for (uint64_t i = 0; i < n; i++) if (d[i].vt) d[i].box->count++;
}
static Pool *box_mut(Dyn *d) {
    DynBox *bx = d->box;
    if (__builtin_expect(!D1_UNIQ(bx), 0)) {
        DynBox *nb = xmalloc(sizeof *nb);
        nb->count = 1; nb->payload = bx->payload;
        buf_retain1(nb->payload.sparse); buf_retain1(nb->payload.packed); buf_retain1(nb->payload.dense);
        bx->count--;
        d->box = bx = nb;
    }
    return &bx->payload;
}

static void pool_remove(Pool *p, uint32_t e) { /* swap_remove, as D1/w1.c */
    Buf *sp = p->sparse;
    if (e >= sp->len) return;
    uint32_t i = EL(sp, uint32_t)[e];
    if (i == NONE) return;
    Buf *dn = buf_mut(&p->dense, sizeof(Comp8), NULL);
    if (i >= dn->len) d1_panic("index out of bounds");
    EL(dn, Comp8)[i] = EL(dn, Comp8)[dn->len - 1];
    dn->len--; dn->shifts++;
    Buf *pk = buf_mut(&p->packed, 4, NULL);
    if (i >= pk->len) d1_panic("index out of bounds");
    EL(pk, uint32_t)[i] = EL(pk, uint32_t)[pk->len - 1];
    pk->len--; pk->shifts++;
    sp = buf_mut(&p->sparse, 4, NULL);
    if (i < pk->len) {
        uint32_t moved = EL(pk, uint32_t)[i];
        if (moved >= sp->len) d1_panic("index out of bounds");
        EL(sp, uint32_t)[moved] = i;
    }
    EL(sp, uint32_t)[e] = NONE;
}
static void pool_swap(Pool *p, uint32_t i, uint32_t j) { /* EnTT's swap_elements, for the group */
    if (i == j) return;
    Buf *dn = buf_mut(&p->dense, sizeof(Comp8), NULL);
    Buf *pk = buf_mut(&p->packed, 4, NULL);
    Buf *sp = buf_mut(&p->sparse, 4, NULL);
    if (i >= dn->len || j >= dn->len || i >= pk->len || j >= pk->len) d1_panic("index out of bounds");
    Comp8 t = EL(dn, Comp8)[i]; EL(dn, Comp8)[i] = EL(dn, Comp8)[j]; EL(dn, Comp8)[j] = t;
    uint32_t a = EL(pk, uint32_t)[i], b = EL(pk, uint32_t)[j];
    EL(pk, uint32_t)[i] = b; EL(pk, uint32_t)[j] = a;
    if (a >= sp->len || b >= sp->len) d1_panic("index out of bounds");
    EL(sp, uint32_t)[b] = i; EL(sp, uint32_t)[a] = j;
    dn->shifts++; pk->shifts++;
}
static const PoolVT VT_POS = {0, pool_remove};
static const PoolVT VT_VEL = {1, pool_remove};

static Pool *reg_pool(Registry *r, uint32_t ti, const PoolVT *vt) { /* as D1/w1.c */
    Buf *pb = buf_mut(&r->pools, sizeof(Dyn), dyn_retain);
    while (pb->len <= ti) {
        PUSH(&r->pools, Dyn, ((Dyn){NULL, NULL}), dyn_retain);
        pb = r->pools;
    }
    Dyn *d = &EL(pb, Dyn)[ti];
    if (d->vt == NULL) {
        DynBox *bx = xmalloc(sizeof *bx);
        bx->count = 1; bx->payload = (Pool){&EMPTY, &EMPTY, &EMPTY};
        d->vt = vt; d->box = bx;
    }
#ifndef D1_PROVEN
    if (d->vt != vt) d1_panic("cast: not this pool type"); /* D1_PROVEN: type numbers are constants */
#endif
    return box_mut(d);
}
static const Pool *reg_pool_read(const Registry *r, uint32_t ti, const PoolVT *vt) {
    const Buf *pb = r->pools;
    if (ti >= pb->len) d1_panic("index out of bounds");
    const Dyn *d = &EL(pb, Dyn)[ti];
    if (d->vt != vt) d1_panic("cast: not this pool type");
    return &d->box->payload;
}
static inline int pool_has(const Pool *p, uint32_t e) {
    return e < p->sparse->len && EL(p->sparse, uint32_t)[e] != NONE;
}
static uint32_t reg_create(Registry *r) { return r->next++; }

static void group_maybe_enter(Registry *r, uint32_t e) { /* the group handler, on construct */
    Pool *pa = reg_pool(r, 0, &VT_POS), *pb = reg_pool(r, 1, &VT_VEL);
    if (!pool_has(pa, e) || !pool_has(pb, e)) return;
    uint32_t g = (uint32_t)r->glen;
    pool_swap(pa, EL(pa->sparse, uint32_t)[e], g);
    pool_swap(pb, EL(pb->sparse, uint32_t)[e], g);
    r->glen++;
}
static void reg_emplace(Registry *r, uint32_t ti, const PoolVT *vt, uint32_t e, Comp8 v) {
    Pool *p = reg_pool(r, ti, vt);
    Buf *sp = p->sparse;
    if (e >= sp->len) sp = buf_resize_u32(&p->sparse, (uint64_t)e + 1, NONE);
    else sp = buf_mut(&p->sparse, 4, NULL);
    if (EL(sp, uint32_t)[e] != NONE) d1_panic("entity already has this component");
    EL(sp, uint32_t)[e] = (uint32_t)p->dense->len;
    PUSH(&p->packed, uint32_t, e, NULL);
    PUSH(&p->dense, Comp8, v, NULL);
    group_maybe_enter(r, e);
}
static void reg_destroy(Registry *r, uint32_t e) {
    Pool *pa = reg_pool(r, 0, &VT_POS), *pb = reg_pool(r, 1, &VT_VEL);
    if (pool_has(pa, e) && pool_has(pb, e) && EL(pa->sparse, uint32_t)[e] < r->glen) { /* on destroy */
        r->glen--;
        uint32_t g = (uint32_t)r->glen;
        pool_swap(pa, EL(pa->sparse, uint32_t)[e], g);
        pool_swap(pb, EL(pb->sparse, uint32_t)[e], g);
    }
    Buf *pl = buf_mut(&r->pools, sizeof(Dyn), dyn_retain);
    for (uint64_t i = 0; i < pl->len; i++) {
        Dyn *d = &EL(pl, Dyn)[i];
        if (d->vt) {
            d->vt->remove(box_mut(d), e); /* a dyn call: not quiet */
            pl = buf_mut(&r->pools, sizeof(Dyn), dyn_retain); /* breaker B6's fix: `self.pools` is fetched again */
        }
    }
}

int main(void) {
    h_start();
    enum { N = 1000000 };
    Registry reg = {&EMPTY, 0, 0};
    for (uint32_t i = 0; i < N; i++) {
        uint32_t e = reg_create(&reg);
        reg_emplace(&reg, 0, &VT_POS, e, (Comp8){(float)(i % 1000), (float)(i / 1000)});
        if (i % 2 == 0) reg_emplace(&reg, 1, &VT_VEL, e, (Comp8){(float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f});
    }
    H_PHASE("setup");
    for (uint32_t s = 0; s < 100; s++) {
        /* `for e, pos, vel in reg.group[Position, Velocity]()`, the generator inlined:
           with a = self.pool[A](), b = self.pool[B](): k = glen-1 .. 0: yield a.packed[k], a.dense[k], b.dense[k] */
        Pool *pa = reg_pool(&reg, 0, &VT_POS), *pb = reg_pool(&reg, 1, &VT_VEL);
        Buf *da = buf_mut(&pa->dense, sizeof(Comp8), NULL); /* pos is written: unique, hoisted */
        Buf *pk = pa->packed, *db = pb->dense;
        int64_t k = (int64_t)reg.glen - 1;
        if (k >= 0 && ((uint64_t)k >= da->len || (uint64_t)k >= pk->len || (uint64_t)k >= db->len))
            d1_panic("index out of bounds");                  /* the cursor's range, hoisted */
        for (; k >= 0; k--) {
#ifdef NOINLINE
            /* an out-of-line generator resumes here: D1-6 fetches its projections again */
            pa = reg_pool(&reg, 0, &VT_POS); pb = reg_pool(&reg, 1, &VT_VEL);
            da = buf_mut(&pa->dense, sizeof(Comp8), NULL); pk = pa->packed; db = pb->dense;
            if ((uint64_t)k >= da->len || (uint64_t)k >= pk->len || (uint64_t)k >= db->len) d1_panic("index out of bounds");
#endif
            uint32_t e = EL(pk, uint32_t)[k];
            Position *pos = (Position *)&EL(da, Comp8)[k];
            const Velocity *vel = (const Velocity *)&EL(db, Comp8)[k];
            pos->x += vel->dx;
            pos->y += vel->dy;
            if (e % 100 == s) {
                reg_destroy(&reg, e);
                uint32_t n = reg_create(&reg);
                reg_emplace(&reg, 0, &VT_POS, n, (Comp8){0.0f, 0.0f});
                reg_emplace(&reg, 1, &VT_VEL, n, (Comp8){1.0f, 1.0f});
                /* reg changed: fetch the projections again, test uniqueness again, re-check k - 1 */
                pa = reg_pool(&reg, 0, &VT_POS); pb = reg_pool(&reg, 1, &VT_VEL);
                da = buf_mut(&pa->dense, sizeof(Comp8), NULL); pk = pa->packed; db = pb->dense;
                if (k > 0 && ((uint64_t)(k - 1) >= da->len || (uint64_t)(k - 1) >= pk->len || (uint64_t)(k - 1) >= db->len))
                    d1_panic("index out of bounds");
            }
        }
    }
    H_PHASE("steps");
    const Pool *pp = reg_pool_read(&reg, 0, &VT_POS);
    const Buf *dp = pp->dense;
    uint64_t sum = 0;
    for (uint64_t i = 0; i < dp->len; i++) {
        const Position *q = (const Position *)&EL(dp, Comp8)[i];
        sum += (uint64_t)((int64_t)(q->x * 4.0f) + (int64_t)(q->y * 4.0f));
    }
    sum += pp->packed->len;
#ifdef NOINLINE
    h_report("d1-w1-group-noinline", sum);
#else
    h_report("d1-w1-group", sum);
#endif
    return 0;
}
