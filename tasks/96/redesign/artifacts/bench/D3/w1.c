/* D3 model of W1 (ECS), as the D3 compiler would emit the program in
   tasks/96/redesign/phase3-design-D3.md (section 3, W1).

   What D3 puts here, and nothing else:
   - Registry = EnTT-style sparse sets (std.ecs). Entity ids come from a counter and are never reused,
     so a stale id is always caught by its empty sparse slot (no version word needed).
   - Pools are objects owned by the registry's pool table, indexed by the program's component type
     number (fixed at compile time): no run-time cast check. A pool's header is its type index
     (interface dispatch for destroy); no generation word (no checked watcher of a pool exists).
   - The view loop's body may add and destroy (a static fact), so the loop PINS both pools:
     while pinned a removal leaves a tombstone and an add never moves an element (in place while
     there is capacity, else an overflow page). The loop visits the lead pool's slots that existed
     when it began and skips tombstones. When the last pin is released the pool SETTLES: overflow is
     merged, tombstones are filled swap-and-pop (EnTT order), sparse slots are fixed.
   - The pin is a run-time counter because destroy() reaches the pools through the type-erased table.
   - Bounds checks where the compiler cannot prove the index: sparse lookups, pool element access.
   - The pinned pools' contiguous buffers cannot move while pinned (a language guarantee), so their
     data pointers are loaded once per loop; the sparse lists are not pinned and are reloaded.
   Left out on purpose, identical in every memory model: integer overflow checks ([TYP-8]).
   The pin record's side fields (tombstones, overflow) are inline here; the design keeps them in a
   side record reached only on the slow paths. The fast paths are the same. */
#include "../harness.h"
#include <string.h>

#define N 1000000u
#define STEPS 100
#define NIL 0xFFFFFFFFu /* sparse slot: no component. ents slot: a tombstone */
#define OVP 4096        /* overflow page: elements added while pinned and full */

static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)
static void *xrealloc(void *p, size_t n) { void *q = realloc(p, n); CHECK(q, "out of memory"); return q; }

typedef struct { float x, y; } Position;
typedef struct { float dx, dy; } Velocity;

/* List[u32] in a place that is never pinned: the plain 24-byte header. */
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

typedef struct { uint32_t type; } PoolHead; /* object header: type index only */
enum { T_Position = 0, T_Velocity = 1, NTYPES = 2 }; /* the program's component type numbers */

#define POOL(T)                                                                                    \
typedef struct {                                                                                   \
    uint32_t type;                                                                                 \
    /* dense + entities: pinnable List fields (header + pin word + side fields) */                \
    T *dense; uint32_t *ents; int64_t len, cap;                                                    \
    uint32_t pins;                                                                                 \
    T **ovp; uint32_t **ovp_ents; int64_t ov_len, ov_pcap; /* overflow pages, only while pinned */ \
    int64_t *tombs; int64_t ntombs, tcap;                  /* slots removed while pinned */        \
    ListU32 sparse;                                                                                \
} Pool_##T;                                                                                        \
static Pool_##T *T##_new(void) {                                                                   \
    Pool_##T *p = calloc(1, sizeof *p); CHECK(p, "out of memory"); p->type = T_##T; return p;     \
}                                                                                                  \
static T *T##_at_slow(Pool_##T *p, int64_t j) {                                                    \
    int64_t k = j - p->len;                                                                        \
    CHECK(k >= 0 && k < p->ov_len, "index out of bounds");                                         \
    return &p->ovp[k / OVP][k % OVP];                                                              \
}                                                                                                  \
static inline T *T##_at(Pool_##T *p, int64_t j) {                                                  \
    if (__builtin_expect(j < p->len, 1)) return &p->dense[j];                                      \
    return T##_at_slow(p, j);                                                                      \
}                                                                                                  \
static inline uint32_t *T##_ent_at(Pool_##T *p, int64_t j) {                                       \
    if (__builtin_expect(j < p->len, 1)) return &p->ents[j];                                       \
    int64_t k = j - p->len;                                                                        \
    CHECK(k >= 0 && k < p->ov_len, "index out of bounds");                                         \
    return &p->ovp_ents[k / OVP][k % OVP];                                                         \
}                                                                                                  \
static void T##_add(Pool_##T *p, uint32_t e, T v) {                                                \
    sparse_cover(&p->sparse, e);                                                                   \
    CHECK(p->sparse.data[e] == NIL, "entity already has this component");                         \
    int64_t j;                                                                                     \
    if (p->len < p->cap && p->ov_len == 0) {                                                       \
        j = p->len++; p->dense[j] = v; p->ents[j] = e;                                             \
    } else if (p->pins == 0) {               /* not pinned: grow by moving, as any List does */    \
        int64_t c = p->cap ? p->cap * 2 : 4;                                                       \
        p->dense = xrealloc(p->dense, (size_t)c * sizeof(T));                                      \
        p->ents = xrealloc(p->ents, (size_t)c * 4);                                                \
        p->cap = c;                                                                                \
        j = p->len++; p->dense[j] = v; p->ents[j] = e;                                             \
    } else {                                 /* pinned and full: an overflow page, nothing moves */\
        int64_t k = p->ov_len;                                                                     \
        if (k % OVP == 0) {                                                                        \
            int64_t pg = k / OVP;                                                                  \
            if (pg == p->ov_pcap) {                                                                \
                p->ov_pcap = p->ov_pcap ? p->ov_pcap * 2 : 4;                                      \
                p->ovp = xrealloc(p->ovp, (size_t)p->ov_pcap * sizeof(T *));                       \
                p->ovp_ents = xrealloc(p->ovp_ents, (size_t)p->ov_pcap * sizeof(uint32_t *));      \
            }                                                                                      \
            p->ovp[pg] = xrealloc(NULL, OVP * sizeof(T));                                          \
            p->ovp_ents[pg] = xrealloc(NULL, OVP * 4);                                             \
        }                                                                                          \
        p->ovp[k / OVP][k % OVP] = v; p->ovp_ents[k / OVP][k % OVP] = e;                           \
        p->ov_len++;                                                                               \
        j = p->len + k;                                                                            \
    }                                                                                              \
    p->sparse.data[e] = (uint32_t)j;                                                               \
}                                                                                                  \
static void T##_remove(void *pv, uint32_t e) {                                                     \
    Pool_##T *p = pv;                                                                              \
    if ((int64_t)e >= p->sparse.len) return;                                                       \
    uint32_t j = p->sparse.data[e];                                                                \
    if (j == NIL) return;                                                                          \
    p->sparse.data[e] = NIL;                                                                       \
    if (p->pins) {                           /* pinned: a tombstone; nothing moves until settle */ \
        *T##_ent_at(p, j) = NIL;                                                                   \
        if (p->ntombs == p->tcap) {                                                                \
            p->tcap = p->tcap ? p->tcap * 2 : 64;                                                  \
            p->tombs = xrealloc(p->tombs, (size_t)p->tcap * 8);                                    \
        }                                                                                          \
        p->tombs[p->ntombs++] = j;                                                                 \
    } else {                                 /* swap-and-pop, as EnTT */                           \
        int64_t last = p->len - 1;                                                                 \
        if ((int64_t)j != last) {                                                                  \
            p->dense[j] = p->dense[last]; p->ents[j] = p->ents[last];                              \
            p->sparse.data[p->ents[j]] = j;                                                        \
        }                                                                                          \
        p->len--;                                                                                  \
    }                                                                                              \
}                                                                                                  \
static void T##_settle(Pool_##T *p) {                                                              \
    if (p->ov_len) {                         /* merge overflow pages behind the contiguous part */ \
        int64_t need = p->len + p->ov_len, c = p->cap ? p->cap : 4;                                \
        while (c < need) c *= 2;                                                                   \
        p->dense = xrealloc(p->dense, (size_t)c * sizeof(T));                                      \
        p->ents = xrealloc(p->ents, (size_t)c * 4);                                                \
        p->cap = c;                                                                                \
        for (int64_t k = 0; k < p->ov_len; k++) {                                                  \
            p->dense[p->len + k] = p->ovp[k / OVP][k % OVP];                                       \
            p->ents[p->len + k] = p->ovp_ents[k / OVP][k % OVP];                                   \
        }                                                                                          \
        for (int64_t pg = 0; pg * OVP < p->ov_len; pg++) { free(p->ovp[pg]); free(p->ovp_ents[pg]); } \
        p->len = need; p->ov_len = 0;                                                              \
    }                                                                                              \
    for (int64_t t = 0; t < p->ntombs; t++) {                                                      \
        int64_t j = p->tombs[t];                                                                   \
        while (p->len > 0 && p->ents[p->len - 1] == NIL) p->len--;                                 \
        if (j < p->len) {                                                                          \
            int64_t last = p->len - 1;                                                             \
            p->dense[j] = p->dense[last]; p->ents[j] = p->ents[last];                              \
            p->sparse.data[p->ents[j]] = (uint32_t)j;                                              \
            p->len--;                                                                              \
        }                                                                                          \
    }                                                                                              \
    p->ntombs = 0;                                                                                 \
}                                                                                                  \
static inline void T##_unpin(Pool_##T *p) {                                                        \
    if (--p->pins == 0 && (p->ntombs | p->ov_len)) T##_settle(p);                                  \
}

POOL(Position)
POOL(Velocity)

/* interface AnyPool: the method table indexed by the object's type index */
typedef struct { void (*remove)(void *, uint32_t); } AnyPoolVT;
static const AnyPoolVT ANYPOOL[NTYPES] = { { Position_remove }, { Velocity_remove } };

typedef struct {
    void *pools[NTYPES];  /* List[AnyPool] sized by the compiler to the program's component types */
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

/* The user's loop body, written once:
       pos.x += vel.dx; pos.y += vel.dy
       if e.id() % 100 == s: reg.destroy(e); n = reg.create(); reg.emplace(n, Position(0, 0)); reg.emplace(n, Velocity(1, 1)) */
static inline void body(Registry *r, uint32_t e, Position *pos, Velocity *vel, uint32_t s) {
    pos->x += vel->dx;
    pos->y += vel->dy;
    if (e % 100u == s) {
        reg_destroy(r, e);
        uint32_t n = reg_create(r);
        Position_add(reg_pool_Position(r), n, (Position){0.0f, 0.0f});
        Velocity_add(reg_pool_Velocity(r), n, (Velocity){1.0f, 1.0f});
    }
}

/* `for e, pos, vel in reg.view[(Position, Velocity)]()`: EnTT leads with the smaller pool. The
   compiler emits one loop per lead; both pools are pinned for the loop. */
#define VIEW_LOOP(LT, OT, POSEXPR, VELEXPR)                                                       \
    {                                                                                              \
        Pool_##LT *lp = (Pool_##LT *)r->pools[T_##LT];                                             \
        Pool_##OT *op = (Pool_##OT *)r->pools[T_##OT];                                             \
        lp->pins++; op->pins++;                                                                    \
        int64_t n0 = lp->len + lp->ov_len;   /* the slots that exist when the loop begins */       \
        LT *l_dense = lp->dense; uint32_t *l_ents = lp->ents; int64_t l_main = lp->len;            \
        OT *o_dense = op->dense;             /* pinned: cannot move during the loop */             \
        for (int64_t i = 0; i < n0; i++) {                                                         \
            uint32_t e; LT *lv;                                                                    \
            if (__builtin_expect(i < l_main, 1)) { e = l_ents[i]; lv = &l_dense[i]; }              \
            else { e = *LT##_ent_at(lp, i); lv = LT##_at(lp, i); }                                 \
            if (e == NIL) continue;                         /* tombstone */                        \
            if ((int64_t)e >= op->sparse.len) continue;     /* sparse list: reloaded */            \
            uint32_t j = op->sparse.data[e];                                                       \
            if (j == NIL) continue;                                                                \
            OT *ov = __builtin_expect((int64_t)j < op->len, 1) ? &o_dense[j] : OT##_at_slow(op, j);\
            body(r, e, POSEXPR, VELEXPR, s);                                                       \
        }                                                                                          \
        LT##_unpin(lp); OT##_unpin(op);                                                            \
    }

static void step(Registry *r, uint32_t s) {
    Pool_Position *pp = reg_pool_Position(r);
    Pool_Velocity *vp = reg_pool_Velocity(r);
    int64_t np = pp->len + pp->ov_len - pp->ntombs, nv = vp->len + vp->ov_len - vp->ntombs;
    if (nv <= np) VIEW_LOOP(Velocity, Position, ov, lv)
    else VIEW_LOOP(Position, Velocity, lv, ov)
}

int main(void) {
    h_start();
    Registry *r = calloc(1, sizeof *r); /* a local object: its home is main's frame */
    CHECK(r, "out of memory");
    for (uint32_t i = 0; i < N; i++) {
        uint32_t e = reg_create(r);
        Position_add(reg_pool_Position(r), e, (Position){(float)(i % 1000), (float)(i / 1000)});
        if (i % 2 == 0)
            Velocity_add(reg_pool_Velocity(r), e, (Velocity){(float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f});
    }
    for (uint32_t s = 0; s < STEPS; s++) step(r, s);

    /* checksum: for e, pos in reg.view[(Position,)](): sum += int(pos.x * 4.0) + int(pos.y * 4.0);
       plus len(reg.storage[Position]()). The loop changes nothing: no pin, no tombstone test. */
    Pool_Position *pp = reg_pool_Position(r);
    uint64_t sum = 0;
    for (int64_t i = 0; i < pp->len; i++)
        sum += (uint64_t)((int64_t)(pp->dense[i].x * 4.0f) + (int64_t)(pp->dense[i].y * 4.0f));
    sum += (uint64_t)pp->len;
    h_report("D3-w1", sum);
    return 0;
}
