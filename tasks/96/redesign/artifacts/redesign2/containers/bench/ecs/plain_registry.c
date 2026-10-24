/* Plain C VARIANT with the library structure of ember_registry.c and lastnight_D3.c (pools are objects
   in the registry's table, destroy reaches every pool through a method table, the view loop holds the
   pools' arrays in locals and walks the lead pool backwards), with no checks and no cursor rule: what
   that structure costs by itself, against plain_sparse.c (direct calls). */
#include "../harness.h"
#include <string.h>

#define N 1000000u
#define STEPS 100
#define NIL 0xFFFFFFFFu /* sparse slot: no component */

static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) ((void)0)   /* plain C: no checks */
static void *xrealloc(void *p, size_t n) { void *q = realloc(p, n); CHECK(q, "out of memory"); return q; }

typedef struct { float x, y; } Position;
typedef struct { float dx, dy; } Velocity;

typedef struct { uint32_t *data; int64_t len, cap; } ListU32;

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

typedef struct { uint32_t type; } PoolHead;
enum { T_Position = 0, T_Velocity = 1, NTYPES = 2 };

#define POOL(T)                                                                                    \
typedef struct {                                                                                   \
    uint32_t type;                                                                                 \
    T *dense; uint32_t *ents; int64_t len, cap;                                                    \
    int64_t cursor;                          /* the running loop's position; 0 when none */        \
    ListU32 sparse;                                                                                \
} Pool_##T;                                                                                        \
static Pool_##T *T##_new(void) {                                                                   \
    Pool_##T *p = calloc(1, sizeof *p); CHECK(p, "out of memory"); p->type = T_##T; return p;     \
}                                                                                                  \
static void T##_add(Pool_##T *p, uint32_t e, T v) {                                                \
    sparse_cover(&p->sparse, e);                                                                   \
    CHECK(p->sparse.data[e] == NIL, "entity already has this component");                         \
    if (p->len == p->cap) {                  /* grow by moving, as any List does */                \
        int64_t c = p->cap ? p->cap * 2 : 4;                                                       \
        p->dense = xrealloc(p->dense, (size_t)c * sizeof(T));                                      \
        p->ents = xrealloc(p->ents, (size_t)c * 4);                                                \
        p->cap = c;                                                                                \
    }                                                                                              \
    int64_t j = p->len++; p->dense[j] = v; p->ents[j] = e;                                         \
    p->sparse.data[e] = (uint32_t)j;                                                               \
}                                                                                                  \
static void T##_remove(void *pv, uint32_t e) {                                                     \
    Pool_##T *p = pv;                                                                              \
    if ((int64_t)e >= p->sparse.len) return;                                                       \
    uint32_t j = p->sparse.data[e];                                                                \
    if (j == NIL) return;                                                                          \
    CHECK((int64_t)j < p->len, "index out of bounds");                                             \
    p->sparse.data[e] = NIL;                                                                       \
    if (0) {   /* plain C: no cursor rule */                                         \
        int64_t c = --p->cursor;                                                                   \
        p->dense[j] = p->dense[c]; p->ents[j] = p->ents[c];                                        \
        p->sparse.data[p->ents[j]] = j;                                                            \
        j = (uint32_t)c;                                                                           \
    }                                                                                              \
    int64_t last = p->len - 1;               /* swap-and-pop, as EnTT */                           \
    if ((int64_t)j != last) {                                                                      \
        p->dense[j] = p->dense[last]; p->ents[j] = p->ents[last];                                  \
        p->sparse.data[p->ents[j]] = j;                                                            \
    }                                                                                              \
    p->len--;                                                                                      \
}

POOL(Position)
POOL(Velocity)

typedef struct { void (*remove)(void *, uint32_t); } AnyPoolVT;
static const AnyPoolVT ANYPOOL[NTYPES] = { { Position_remove }, { Velocity_remove } };

typedef struct {
    void *pools[NTYPES];
    uint32_t next_id;
} Registry;

static Pool_Position *reg_pool_Position(Registry *r) {
    if (!r->pools[T_Position]) r->pools[T_Position] = Position_new();
    return r->pools[T_Position];
}
static Pool_Velocity *reg_pool_Velocity(Registry *r) {
    if (!r->pools[T_Velocity]) r->pools[T_Velocity] = Velocity_new();
    return r->pools[T_Velocity];
}
static uint32_t reg_create(Registry *r) {
    CHECK(r->next_id != NIL, "entity ids exhausted");
    return r->next_id++;
}
static void reg_destroy(Registry *r, uint32_t e) {
    for (int t = 0; t < NTYPES; t++) {
        void *p = r->pools[t];
        if (p) ANYPOOL[((PoolHead *)p)->type].remove(p, e);
    }
}

/* `for e, pos, vel in reg.view[(Position, Velocity)]()`: EnTT leads with the smaller pool and walks
   it backwards. The body is the user's, inlined. */
#define VIEW_LOOP(LT, OT, POSEXPR, VELEXPR)                                                       \
    {                                                                                              \
        Pool_##LT *lp = (Pool_##LT *)r->pools[T_##LT];                                             \
        Pool_##OT *op = (Pool_##OT *)r->pools[T_##OT];                                             \
        /* the loop's view of the pools, valid until the rare branch changes them */              \
        LT *l_dense = lp->dense; uint32_t *l_ents = lp->ents;                                      \
        uint32_t *o_sparse = op->sparse.data; int64_t o_slen = op->sparse.len;                     \
        OT *o_dense = op->dense; int64_t o_len = op->len;                                          \
        for (int64_t i = lp->len; i-- > 0;) {                                                      \
            uint32_t e = l_ents[i]; LT *lv = &l_dense[i];                                          \
            if ((int64_t)e >= o_slen) continue;                                                    \
            uint32_t j = o_sparse[e];                                                              \
            if ((int64_t)j >= o_len) continue;      /* NIL is >= len */                            \
            OT *ov = &o_dense[j];                                                                  \
            Position *pos = POSEXPR; Velocity *vel = VELEXPR;                                      \
            pos->x += vel->dx;                                                                     \
            pos->y += vel->dy;                                                                     \
            if (e % 100u == s) {                                                                   \
                reg_destroy(r, e);                                                                 \
                uint32_t n = reg_create(r);                                                        \
                Position_add(reg_pool_Position(r), n, (Position){0.0f, 0.0f});                     \
                Velocity_add(reg_pool_Velocity(r), n, (Velocity){1.0f, 1.0f});                     \
                l_dense = lp->dense; l_ents = lp->ents;                                            \
                o_sparse = op->sparse.data; o_slen = op->sparse.len;                               \
                o_dense = op->dense; o_len = op->len;                                              \
            }                                                                                      \
        }                                                                                          \
    }

static void step(Registry *r, uint32_t s) {
    Pool_Position *pp = reg_pool_Position(r);
    Pool_Velocity *vp = reg_pool_Velocity(r);
    if (vp->len <= pp->len) VIEW_LOOP(Velocity, Position, ov, lv)
    else VIEW_LOOP(Position, Velocity, lv, ov)
}

int main(void) {
    h_start();
    Registry *r = calloc(1, sizeof *r);
    CHECK(r, "out of memory");
    for (uint32_t i = 0; i < N; i++) {
        uint32_t e = reg_create(r);
        Position_add(reg_pool_Position(r), e, (Position){(float)(i % 1000), (float)(i / 1000)});
        if (i % 2 == 0)
            Velocity_add(reg_pool_Velocity(r), e, (Velocity){(float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f});
    }
    for (uint32_t s = 0; s < STEPS; s++) step(r, s);

    Pool_Position *pp = reg_pool_Position(r);
    uint64_t sum = 0;
    for (int64_t i = 0; i < pp->len; i++)
        sum += (uint64_t)((int64_t)(pp->dense[i].x * 4.0f) + (int64_t)(pp->dense[i].y * 4.0f));
    sum += (uint64_t)pp->len;
    h_report("plain-w1-registry", sum);
    return 0;
}
