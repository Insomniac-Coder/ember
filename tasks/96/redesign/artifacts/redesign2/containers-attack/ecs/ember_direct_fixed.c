/* COPY of ../../containers/bench/ecs/ember_direct.c with one change in del(): the guard `if (i != last)`
   (the one ember_registry.c already has). cursor_check2.c shows the original del() breaks the pool when
   one step destroys the current entity while it is the last and then an entity not yet reached.
   W1 (ECS, EnTT layout) as the answer of phase3b-containers.md compiles it, written with the same
   program structure as plain_sparse.c (the pools are called directly), so the difference between the
   two is the memory model's cost alone.

   The Ember program (W1 in phase3-design-D3.md, unchanged):
       for s in 0..100:
           for e, pos, vel in reg.view[(Position, Velocity)]():
               pos.x += vel.dx
               pos.y += vel.dy
               if e.id() % 100 == s:
                   reg.destroy(e)
                   n = reg.create()
                   reg.emplace(n, Position(x=0.0, y=0.0))
                   reg.emplace(n, Velocity(dx=1.0, dy=1.0))

   What the answer puts here, and nothing else:
   - Each Array is Ember's {data, len, cap}; a pool is three Arrays (sparse, ents, dense) and a
     cursor. No pins, no tombstones, no pages, no generations.
   - The view loop is std.ecs library code: it walks the smaller pool (Velocity) backwards by
     position, as EnTT does, and probes Position through `sparse`. `pos` and `vel` are names of places
     keyed by the entity (Position of e, Velocity of e). The rule "an address is kept while nothing can
     change its container, and read again after something that may" is applied to the loop's own
     state too: the arrays' data pointers and lengths are held in locals on the common path (no call
     there can change a pool) and read again after the rare branch, the only code that changes pools.
   - Bounds checks: on the common path the position j only decreases from a length already checked,
     so it needs none; after the rare branch the position read back from the cursor is checked once
     against both lengths. pos.sparse[id] and pos.dense[pi] are guarded by the library's own `if`
     tests (the same tests plain C makes). Every index inside del/add is checked.
   - Before the rare branch's calls the loop publishes its position in the pool's `cursor` and reads
     it back after; `del` compares the removed position with the cursor, so removing an entity the loop
     has not reached yet keeps "every entity once" (two moves instead of one). Destroying the current
     entity, as W1 does, is EnTT's swap-and-pop.
   - Integer overflow checks are left out, as in every phase 3 model and in plain C.
   The first version (ember_direct_naive.c) read every field through the pool struct each iteration
   and checked j twice per iteration; it was about 20% slower than plain C. */
#include "../harness.h"
#include <string.h>
#define N 1000000
#define NONE 0xFFFFFFFFu

static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c) do { if (__builtin_expect(!(c), 0)) panic("index out of bounds"); } while (0)

typedef struct { float x, y; } V2;
typedef struct { uint32_t *data; size_t len, cap; } ArrU32;
typedef struct { V2 *data; size_t len, cap; } ArrV2;
typedef struct { ArrU32 sparse, ents; ArrV2 dense; size_t cursor; } Pool;

static void pushu(ArrU32 *a, uint32_t v) {
    if (a->len == a->cap) { a->cap = a->cap ? a->cap * 2 : 4; a->data = realloc(a->data, a->cap * 4); }
    a->data[a->len++] = v;
}
static void pushv(ArrV2 *a, V2 v) {
    if (a->len == a->cap) { a->cap = a->cap ? a->cap * 2 : 4; a->data = realloc(a->data, a->cap * sizeof(V2)); }
    a->data[a->len++] = v;
}
static void grow_sparse(ArrU32 *s, uint32_t id) {        /* sparse.resize(id + 1, NONE), doubling */
    if (id < s->len) return;
    size_t nc = s->cap ? s->cap : 1024;
    while (nc <= id) nc *= 2;
    if (nc > s->cap) { s->data = realloc(s->data, nc * 4); s->cap = nc; }
    for (size_t i = s->len; i < nc; i++) s->data[i] = NONE;
    s->len = nc;
}
static void add(Pool *p, uint32_t id, V2 v) {
    grow_sparse(&p->sparse, id);
    p->sparse.data[id] = (uint32_t)p->ents.len;
    pushu(&p->ents, id);
    pushv(&p->dense, v);
}
static void del(Pool *p, uint32_t id) {
    CHECK(id < p->sparse.len);
    uint32_t i = p->sparse.data[id];
    CHECK(i < p->ents.len && i < p->dense.len);
    if (__builtin_expect(i < p->cursor, 0)) {
        /* an entity the running loop has not reached: keep [0, cursor) = not yet visited */
        size_t c = --p->cursor;                       /* the last not-yet-visited position */
        uint32_t mc = p->ents.data[c];
        p->ents.data[i] = mc; p->dense.data[i] = p->dense.data[c]; p->sparse.data[mc] = i;
        i = (uint32_t)c;                              /* now remove position c as a visited one */
    }
    size_t last = p->ents.len - 1;
    if (i != last) {                                  /* FIX (containers attack): when the current entity was
                                                         the last and is already gone, c == last and the move
                                                         below would point sparse[mc] back at c */
        uint32_t moved = p->ents.data[last];
        p->ents.data[i] = moved; p->dense.data[i] = p->dense.data[last];
        p->sparse.data[moved] = i;
    }
    p->sparse.data[id] = NONE;
    p->ents.len--; p->dense.len--;
}

int main(void) {
    h_start();
    Pool pos = {0}, vel = {0};
    for (uint32_t i = 0; i < N; i++) {
        add(&pos, i, (V2){(float)(i % 1000), (float)(i / 1000)});
        if (i % 2 == 0) add(&vel, i, (V2){(float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f});
    }
    uint32_t next = N;
    for (uint32_t s = 0; s < 100; s++) {
        size_t saved = vel.cursor;                    /* loops nest: keep the outer loop's cursor */
        /* the loop's view of the pools, valid until the rare branch changes them */
        uint32_t *v_ents = vel.ents.data; V2 *v_dense = vel.dense.data;
        uint32_t *p_sparse = pos.sparse.data; size_t p_slen = pos.sparse.len;
        V2 *p_dense = pos.dense.data; size_t p_dlen = pos.dense.len;
        size_t j = vel.ents.len;
        CHECK(j <= vel.dense.len);
        while (j-- > 0) {
            uint32_t id = v_ents[j];
            if (id >= p_slen) continue;               /* the view's own tests (EnTT's contains) */
            uint32_t pi = p_sparse[id];
            if (pi >= p_dlen) continue;               /* NONE is >= len: one test for both */
            V2 *p = &p_dense[pi], *v = &v_dense[j];   /* the names pos and vel */
            p->x += v->x; p->y += v->y;
            if (id % 100 == s) {
                vel.cursor = j;                       /* publish before calls that may destroy */
                del(&pos, id); del(&vel, id);         /* reg.destroy(e) */
                uint32_t e = next++;                  /* reg.create() */
                add(&pos, e, (V2){0, 0}); add(&vel, e, (V2){1, 1});
                j = vel.cursor;                       /* read back: a removal may have moved it */
                CHECK(j <= vel.ents.len && j <= vel.dense.len);
                v_ents = vel.ents.data; v_dense = vel.dense.data;   /* the pools may have moved */
                p_sparse = pos.sparse.data; p_slen = pos.sparse.len;
                p_dense = pos.dense.data; p_dlen = pos.dense.len;
            }
        }
        vel.cursor = saved;
    }
    uint64_t sum = 0;
    for (size_t j = 0; j < pos.dense.len; j++)
        sum += (uint64_t)((int64_t)(pos.dense.data[j].x * 4.0f) + (int64_t)(pos.dense.data[j].y * 4.0f));
    h_report("ecs-ember-direct-fixed", sum + pos.dense.len);
    return 0;
}
