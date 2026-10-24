/* D1 (values and projections), workload W1: an EnTT-style ECS.
   Report: tasks/96/redesign/phase3-design-D1.md (sections 3, 4, 5).

   This is the C that D1's compiler would emit for the W1 program of the report, with the
   std.ecs library (Registry, Pool[T], view) inlined where the report says it is.
   What D1 needs and is therefore here:
   - an Array value is one pointer to a counted buffer (header: count, shift version, len, cap);
     every in-place change first tests count == 1 (copy-on-write), hoisted out of a loop whose
     body cannot share the buffer and not repeated after a call that leaves it unique;
   - every index the compiler cannot prove in range is checked; an induction variable's range
     is checked once before its loop and again after a call that may change the array;
   - a `dyn AnyPool` value is (type table, counted box); `cast p as Pool[T]` on a place
     compares the table; a mutable projection of the box tests the box's count;
   - destroy() calls each pool's `remove` through the type table (EnTT's type-erased remove);
   - swap_remove bumps the buffer's shift version (D1's stale-projection check reads it);
   - the view's projections and cached buffers are fetched again after the loop body changed
     the registry (the destroy branch), and only there.
   Left out on purpose, the same in every memory model: the integer overflow checks of [TYP-8].
   No teardown at the end: memory-only values whose last use ends main are not freed
   (freeing them cannot be observed). */
#include "../harness.h"
#include <string.h>

static _Noreturn void d1_panic(const char *msg) {
    fprintf(stderr, "panic: %s\n", msg);
    fflush(stderr);
    ExitProcess(101);
}
static void *xmalloc(size_t n) {
    void *p = malloc(n);
    if (!p) d1_panic("out of memory");
    return p;
}

/* ---- D1 runtime: counted buffers (Array) ---- */
typedef struct Buf { uint32_t count; uint32_t shifts; uint64_t len; uint64_t cap; } Buf; /* 24 bytes */
static Buf EMPTY = {0x80000000u, 0, 0, 0}; /* the shared empty buffer: never unique, never freed */
#define EL(b, T) ((T *)((Buf *)(b) + 1))
typedef void (*RetainFn)(void *elems, uint64_t n);

/* Make *slot unique (copy-on-write) with room for `need` elements; doubling from 4 ([HEAP-2]). */
static Buf *buf_reserve_slow(Buf **slot, size_t esz, uint64_t need, RetainFn retain) {
    Buf *b = *slot;
    uint64_t cap = b->cap;
    if (need > cap) {
        uint64_t nc = cap < 4 ? 4 : cap * 2;
        while (nc < need) nc *= 2;
        cap = nc;
    }
    if (b->count == 1) {
        b = realloc(b, sizeof(Buf) + cap * esz);
        if (!b) d1_panic("out of memory");
        b->cap = cap;
    } else {
        Buf *nb = xmalloc(sizeof(Buf) + cap * esz);
        nb->count = 1; nb->shifts = 0; nb->len = b->len; nb->cap = cap;
        memcpy(nb + 1, b + 1, b->len * esz);
        if (retain) retain(nb + 1, b->len);
        if (b != &EMPTY) b->count--; /* still shared by someone else: never reaches 0 here */
        b = nb;
    }
    *slot = b;
    return b;
}
static inline Buf *buf_mut(Buf **slot, size_t esz, RetainFn retain) { /* before an in-place write */
    Buf *b = *slot;
    if (__builtin_expect(b->count != 1, 0)) b = buf_reserve_slow(slot, esz, b->len, retain);
    return b;
}
#define PUSH(slot, T, val, retain)                                                                   \
    do {                                                                                             \
        Buf *b_ = *(slot);                                                                           \
        if (__builtin_expect(b_->count != 1 || b_->len == b_->cap, 0))                               \
            b_ = buf_reserve_slow((slot), sizeof(T), b_->len + 1, (retain));                         \
        EL(b_, T)[b_->len++] = (val);                                                                \
    } while (0)

/* Array[u32].resize(n, fill) */
static Buf *buf_resize_u32(Buf **slot, uint64_t n, uint32_t fill) {
    Buf *b = *slot;
    if (b->count != 1 || n > b->cap) b = buf_reserve_slow(slot, 4, n, NULL);
    for (uint64_t i = b->len; i < n; i++) EL(b, uint32_t)[i] = fill;
    if (n > b->len) b->len = n;
    return b;
}

/* ---- the program's types ---- */
typedef struct { float x, y; } Position;
typedef struct { float dx, dy; } Velocity;
/* Pool[Position] and Pool[Velocity] monomorphise to identical code (both 8-byte, 4-aligned);
   one copy over this 8-byte type stands for both. */
typedef struct { float a, b; } Comp8;
#define NONE 0xFFFFFFFFu

typedef struct Pool { Buf *sparse; Buf *packed; Buf *dense; } Pool; /* u32, Entity (u32), Comp8 */
typedef struct PoolVT { uint32_t type_index; void (*remove)(Pool *, uint32_t); } PoolVT;
typedef struct DynBox { uint32_t count; uint32_t pad_; Pool payload; } DynBox; /* counted box of a dyn value */
typedef struct Dyn { const PoolVT *vt; DynBox *box; } Dyn;                    /* vt == NULL: None */
typedef struct Registry { Buf *pools; uint32_t next; } Registry;              /* pools: Array[Option[dyn AnyPool]] */

static void buf_retain1(Buf *b) { if (b != &EMPTY) b->count++; }
static void dyn_retain(void *el, uint64_t n) { /* copying a dyn value shares its box */
    Dyn *d = el;
    for (uint64_t i = 0; i < n; i++) if (d[i].vt) d[i].box->count++;
}
static Pool *box_mut(Dyn *d) { /* mutable projection of a dyn value's payload: unique box first */
    DynBox *bx = d->box;
    if (__builtin_expect(bx->count != 1, 0)) {
        DynBox *nb = xmalloc(sizeof *nb);
        nb->count = 1; nb->payload = bx->payload;
        buf_retain1(nb->payload.sparse); buf_retain1(nb->payload.packed); buf_retain1(nb->payload.dense);
        bx->count--;
        d->box = bx = nb;
    }
    return &bx->payload;
}

/* Pool[T].remove(mut self, e): `if not contains(e): return; i = slot(e); dense.swap_remove(i);
   packed.swap_remove(i); if i < packed.len(): sparse[packed[i]] = i; sparse[e] = NONE` */
static void pool_remove(Pool *p, uint32_t e) {
    Buf *sp = p->sparse;
    if (e >= sp->len) return;
    uint32_t i = EL(sp, uint32_t)[e];
    if (i == NONE) return;
    Buf *dn = buf_mut(&p->dense, sizeof(Comp8), NULL); /* dense.swap_remove(i) */
    if (i >= dn->len) d1_panic("index out of bounds");
    EL(dn, Comp8)[i] = EL(dn, Comp8)[dn->len - 1];
    dn->len--; dn->shifts++;
    Buf *pk = buf_mut(&p->packed, 4, NULL); /* packed.swap_remove(i) */
    if (i >= pk->len) d1_panic("index out of bounds");
    EL(pk, uint32_t)[i] = EL(pk, uint32_t)[pk->len - 1];
    pk->len--; pk->shifts++;
    sp = buf_mut(&p->sparse, 4, NULL);
    if (i < pk->len) {
        uint32_t moved = EL(pk, uint32_t)[i];
        if (moved >= sp->len) d1_panic("index out of bounds");
        EL(sp, uint32_t)[moved] = i;
    }
    EL(sp, uint32_t)[e] = NONE; /* e < sp->len: checked above, sparse unchanged in length since */
}
static const PoolVT VT_POS = {0, pool_remove}; /* type_index[Position]() == 0 */
static const PoolVT VT_VEL = {1, pool_remove}; /* type_index[Velocity]() == 1 */

/* Registry.pool[T](mut self) -> Pool[T]: grows pools to the type's slot, creates the pool on
   first use, then `yield cast p as Pool[T]` (a downcast of a place is a place). */
static Pool *reg_pool(Registry *r, uint32_t ti, const PoolVT *vt) {
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
    if (d->vt != vt) d1_panic("cast: not this pool type");
    return box_mut(d);
}
/* the read-only form (a projection through `self`): no uniqueness needed */
static const Pool *reg_pool_read(const Registry *r, uint32_t ti, const PoolVT *vt) {
    const Buf *pb = r->pools;
    if (ti >= pb->len) d1_panic("index out of bounds");
    const Dyn *d = &EL(pb, Dyn)[ti];
    if (d->vt != vt) d1_panic("cast: not this pool type");
    return &d->box->payload;
}

static uint32_t reg_create(Registry *r) { return r->next++; }

/* Registry.emplace[T](mut self, e, value) */
static void reg_emplace(Registry *r, uint32_t ti, const PoolVT *vt, uint32_t e, Comp8 v) {
    Pool *p = reg_pool(r, ti, vt);
    Buf *sp = p->sparse;
    if (e >= sp->len) sp = buf_resize_u32(&p->sparse, (uint64_t)e + 1, NONE);
    else sp = buf_mut(&p->sparse, 4, NULL);
    if (EL(sp, uint32_t)[e] != NONE) d1_panic("entity already has this component");
    EL(sp, uint32_t)[e] = (uint32_t)p->dense->len;
    PUSH(&p->packed, uint32_t, e, NULL);
    PUSH(&p->dense, Comp8, v, NULL);
}

/* Registry.destroy(mut self, e): `for p in self.pools: match p: Some(pool): pool.remove(e)` */
static void reg_destroy(Registry *r, uint32_t e) {
    Buf *pb = buf_mut(&r->pools, sizeof(Dyn), dyn_retain);
    for (uint64_t i = 0; i < pb->len; i++) {
        Dyn *d = &EL(pb, Dyn)[i];
        if (d->vt) d->vt->remove(box_mut(d), e); /* a dyn call */
    }
}

int main(void) {
    h_start();
    enum { N = 1000000 };
    Registry reg = {&EMPTY, 0};

    for (uint32_t i = 0; i < N; i++) {
        uint32_t e = reg_create(&reg);
        reg_emplace(&reg, 0, &VT_POS, e, (Comp8){(float)(i % 1000), (float)(i / 1000)});
        if (i % 2 == 0) reg_emplace(&reg, 1, &VT_VEL, e, (Comp8){(float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f});
    }

    for (uint32_t s = 0; s < 100; s++) {
        /* view[Position, Velocity], inlined: `with a = self.pool[A](), b = self.pool[B]():` */
        Pool *pa = reg_pool(&reg, 0, &VT_POS);
        Pool *pb = reg_pool(&reg, 1, &VT_VEL);
        Buf *da = buf_mut(&pa->dense, sizeof(Comp8), NULL); /* pos is written: unique, hoisted */
        Buf *sa = pa->sparse, *pk = pb->packed, *db = pb->dense;
        int64_t k = (int64_t)pk->len - 1;                     /* k < packed.len by construction */
        if (k >= 0 && (uint64_t)k >= db->len) d1_panic("index out of bounds"); /* dense[k], hoisted */
        for (; k >= 0; k--) {
            uint32_t e = EL(pk, uint32_t)[k];
            if (e >= sa->len) continue;                       /* a.contains(e) */
            uint32_t idx = EL(sa, uint32_t)[e];
            if (idx == NONE) continue;
            if (idx >= da->len) d1_panic("index out of bounds"); /* a.dense[a.slot(e)] */
            Position *pos = (Position *)&EL(da, Comp8)[idx];
            const Velocity *vel = (const Velocity *)&EL(db, Comp8)[k];
            pos->x += vel->dx;
            pos->y += vel->dy;
            if (e % 100 == s) {
                reg_destroy(&reg, e);
                uint32_t n = reg_create(&reg);
                reg_emplace(&reg, 0, &VT_POS, n, (Comp8){0.0f, 0.0f});
                reg_emplace(&reg, 1, &VT_VEL, n, (Comp8){1.0f, 1.0f});
                /* reg changed: fetch the generator's projections again; the calls left what they
                   changed unique, so no new uniqueness test; re-check the range of k - 1 */
                pa = reg_pool(&reg, 0, &VT_POS);
                pb = reg_pool(&reg, 1, &VT_VEL);
                da = pa->dense; sa = pa->sparse; pk = pb->packed; db = pb->dense;
                if (k > 0 && ((uint64_t)(k - 1) >= pk->len || (uint64_t)(k - 1) >= db->len))
                    d1_panic("index out of bounds");
            }
        }
    }

    /* checksum: `for e, pos in reg.view[Position]()` (read only) */
    const Pool *pp = reg_pool_read(&reg, 0, &VT_POS);
    const Buf *dp = pp->dense;
    uint64_t sum = 0;
    for (uint64_t i = 0; i < dp->len; i++) {
        const Position *q = (const Position *)&EL(dp, Comp8)[i];
        sum += (uint64_t)((int64_t)(q->x * 4.0f) + (int64_t)(q->y * 4.0f));
    }
    sum += pp->packed->len;
    h_report("D1-w1", sum);
    return 0;
}
