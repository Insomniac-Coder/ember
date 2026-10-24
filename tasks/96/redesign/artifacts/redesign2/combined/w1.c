/* combined-w1: W1 (ECS) as the combined memory model's compiler would emit it, in EnTT's full-owning
   group layout. Report: tasks/96/redesign/phase3b-combined.md, section 6.1.
   (Second run of the combined step; the first run's file is first-run/w1.c. What changed: the cursor
   test now lives in the group's one "leave" operation, which destroy() and remove[Velocity]() both
   call, as the containers attack's second pass requires, phase3b-containers-attack.md 7.5; the
   self-check also removes and adds components, and -DNO_CURSOR_RULE shows that it catches a library
   without the rule.)

   The Ember program (the model's syntax):

       for s in range(100):
           for e, pos, vel in reg.group[(Position, Velocity)]():
               pos.x += vel.dx
               pos.y += vel.dy
               if e.id % 100 == s:
                   reg.destroy(e)
                   n = reg.create()
                   reg.add(n, Position(x=0.0, y=0.0))
                   reg.add(n, Velocity(dx=1.0, dy=1.0))

   Layout and library structure are those of plain C (target/redesign2/salvage/plain/w1_group.c):
   one sparse set per component (sparse[id] -> slot, ents[slot] -> id, dense[slot]); the group
   <Position, Velocity> owns both pools, so the entities that have both sit in slots [0, glen) of
   both pools in the same order; the loop runs backwards (EnTT's order). The pools are called
   directly, as in plain C. (A std.ecs registry reaches them through a table with one slot per type,
   the slot number given by the compiler: one load on the rare path, not modelled; references
   attack 3.2.)

   What the model adds to plain C, and nothing else:
   - Arrays are Ember's {ptr, len, cap} with 64-bit len and cap, first growth to 4, then doubling.
   - Bounds checks on every index the compiler cannot prove. On the loop's common path there are
     none: the position k only decreases from a value checked against the three arrays' lengths at
     the loop's start, and nothing on that path can change a pool (effect summaries). After the rare
     branch, which may change the pools, the position is read back and checked once more, and the
     arrays' pointers are read again (names are looked up again after a change).
   - The cursor: the loop's position is stored in the group's `cursor` field only before the calls
     whose effect summaries read it (reg.destroy), and read back after them. Every operation that
     takes an entity out of the walked range goes through group_leave(), which compares the leaving
     entity's slot with the cursor: an entity the loop has not reached yet is first swapped with the
     last unreached slot, so "every entity once" holds (two swaps instead of one; never taken in W1).
     Loops save and restore the outer cursor, so they nest.
   - create() panics when the 32-bit id counter is exhausted (ids are never reused in W1).
   - Integer overflow checks are left out, as in every phase-3 model and in plain C.
   No counts, no generations, no pins, no tombstones: nothing in W1 is a class object.

   Self-check build: -DCURSOR_CHECK (and -DCURSOR_CHECK -DNO_CURSOR_RULE, which must fail). */
#include "../../redesign/bench/harness.h"
#include <string.h>

#define N 1000000u
#define NONE 0xFFFFFFFFu
static __declspec(noreturn) void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)

typedef struct { float x, y; } V2;
typedef struct { uint32_t *ptr; int64_t len, cap; } ArrU32;
typedef struct { V2 *ptr; int64_t len, cap; } ArrV2;
typedef struct { ArrU32 sparse, ents; ArrV2 dense; } Pool;
typedef struct { Pool pos, vel; int64_t glen, cursor; uint32_t next_id; } Registry;

static void push_u32(ArrU32 *a, uint32_t v) {
    if (a->len == a->cap) {
        a->cap = a->cap ? a->cap * 2 : 4;
        a->ptr = realloc(a->ptr, (size_t)a->cap * 4);
        CHECK(a->ptr, "out of memory");
    }
    a->ptr[a->len++] = v;
}
static void push_v2(ArrV2 *a, V2 v) {
    if (a->len == a->cap) {
        a->cap = a->cap ? a->cap * 2 : 4;
        a->ptr = realloc(a->ptr, (size_t)a->cap * sizeof(V2));
        CHECK(a->ptr, "out of memory");
    }
    a->ptr[a->len++] = v;
}
/* sparse.resize(...) so that `id` is covered, filled with NONE (plain C's policy: power of two from 1024) */
static void sparse_cover(ArrU32 *s, uint32_t id) {
    if ((int64_t)id < s->len) return;
    int64_t nc = s->cap ? s->cap : 1024;
    while (nc <= (int64_t)id) nc *= 2;
    s->ptr = realloc(s->ptr, (size_t)nc * 4);
    CHECK(s->ptr, "out of memory");
    for (int64_t i = s->len; i < nc; i++) s->ptr[i] = NONE;
    s->len = s->cap = nc;
}
static inline int pool_has(const Pool *p, uint32_t id) {   /* the library's own tests prove sparse[id] */
    return (int64_t)id < p->sparse.len && p->sparse.ptr[id] != NONE;
}
static void pool_add(Pool *p, uint32_t id, V2 v) {
    sparse_cover(&p->sparse, id);                 /* proves id < sparse.len */
    p->sparse.ptr[id] = (uint32_t)p->ents.len;
    push_u32(&p->ents, id);
    push_v2(&p->dense, v);
}
static void pool_swap(Pool *p, int64_t i, int64_t j) {
    if (i == j) return;
    CHECK(i >= 0 && j >= 0 && i < p->ents.len && j < p->ents.len && i < p->dense.len && j < p->dense.len,
          "index out of bounds");
    uint32_t a = p->ents.ptr[i], b = p->ents.ptr[j];
    V2 t = p->dense.ptr[i];
    p->ents.ptr[i] = b; p->ents.ptr[j] = a;
    p->dense.ptr[i] = p->dense.ptr[j]; p->dense.ptr[j] = t;
    CHECK((int64_t)a < p->sparse.len && (int64_t)b < p->sparse.len, "index out of bounds");
    p->sparse.ptr[b] = (uint32_t)i; p->sparse.ptr[a] = (uint32_t)j;
}
static void pool_del(Pool *p, uint32_t id) {      /* swap-and-pop; caller has tested pool_has */
    int64_t i = p->sparse.ptr[id];
    int64_t last = p->ents.len - 1;
    CHECK(i <= last && last < p->dense.len, "index out of bounds");
    if (i != last) {
        uint32_t moved = p->ents.ptr[last];
        p->ents.ptr[i] = moved;
        p->dense.ptr[i] = p->dense.ptr[last];
        CHECK((int64_t)moved < p->sparse.len, "index out of bounds");
        p->sparse.ptr[moved] = (uint32_t)i;
    }
    p->sparse.ptr[id] = NONE;
    p->ents.len--; p->dense.len--;
}
static void group_enter(Registry *r, uint32_t id) {     /* both pools have id: move it to slot glen */
    pool_swap(&r->pos, r->pos.sparse.ptr[id], r->glen);
    pool_swap(&r->vel, r->vel.sparse.ptr[id], r->glen);
    r->glen++;
}
/* The one operation that takes an entity out of the group's range (the range a running loop walks).
   destroy() and remove[Velocity]() both call it; the cursor rule lives here and nowhere else. */
static void group_leave(Registry *r, uint32_t id) {
    int64_t j = r->pos.sparse.ptr[id];
#ifndef NO_CURSOR_RULE
    if (__builtin_expect(j < r->cursor, 0)) {           /* a running loop has not reached it yet */
        int64_t c = --r->cursor;                        /* the last slot not yet visited */
        pool_swap(&r->pos, j, c); pool_swap(&r->vel, j, c);
        j = c;                                          /* now leave from c, a visited slot */
    }
#endif
    r->glen--;
    pool_swap(&r->pos, j, r->glen); pool_swap(&r->vel, j, r->glen);
}
static void reg_add_pos(Registry *r, uint32_t id, V2 v) { pool_add(&r->pos, id, v); if (pool_has(&r->vel, id)) group_enter(r, id); }
static void reg_add_vel(Registry *r, uint32_t id, V2 v) { pool_add(&r->vel, id, v); if (pool_has(&r->pos, id)) group_enter(r, id); }
static uint32_t reg_create(Registry *r) { CHECK(r->next_id != NONE, "entity ids exhausted"); return r->next_id++; }
static void reg_destroy(Registry *r, uint32_t id) {
    int hp = pool_has(&r->pos, id), hv = pool_has(&r->vel, id);
    if (hp && hv && (int64_t)r->pos.sparse.ptr[id] < r->glen) group_leave(r, id);   /* in_group(r, id) */
    if (hp) pool_del(&r->pos, id);
    if (hv) pool_del(&r->vel, id);
}
#ifdef CURSOR_CHECK
static inline int in_group(const Registry *r, uint32_t id) {
    return pool_has(&r->pos, id) && pool_has(&r->vel, id) && (int64_t)r->pos.sparse.ptr[id] < r->glen;
}
static void reg_remove_vel(Registry *r, uint32_t id) {  /* reg.remove[Velocity](e): the entity lives on */
    if (!pool_has(&r->vel, id)) return;
    if (in_group(r, id)) group_leave(r, id);
    pool_del(&r->vel, id);
}

/* Self-check of the cursor rule (built with -DCURSOR_CHECK; not the timed program): 5,000 random
   loops in which the body destroys the current entity, destroys entities not yet reached and
   entities already visited, takes Velocity away from entities (they leave the group but live on),
   gives Velocity to entities that had only Position (they join the group), and creates entities
   with both. After every change the pools must be consistent and the group must hold exactly the
   entities that have both; every entity in the group when the loop starts and still in it at its
   turn must be visited exactly once; no entity that joined during the loop may be visited.
   Prints "cursor-check ok". With -DNO_CURSOR_RULE it must stop with a "check:" panic. */
static void check_pools(Registry *r) {
    Pool *ps[2] = {&r->pos, &r->vel};
    for (int q = 0; q < 2; q++) {
        Pool *p = ps[q];
        CHECK(p->ents.len == p->dense.len, "check: lengths");
        for (int64_t i = 0; i < p->ents.len; i++) CHECK(p->sparse.ptr[p->ents.ptr[i]] == (uint32_t)i, "check: sparse");
    }
    for (int64_t i = 0; i < r->pos.ents.len; i++) {
        uint32_t e = r->pos.ents.ptr[i];
        int both = pool_has(&r->vel, e);
        CHECK((i < r->glen) == both, "check: group membership");
        if (both) CHECK(r->vel.ents.ptr[i] == e, "check: group order");
    }
}
int main(void) {
    h_start();
    for (uint64_t seed = 1; seed <= 5000; seed++) {
        static Registry reg; Registry *r = &reg;
        free(r->pos.sparse.ptr); free(r->pos.ents.ptr); free(r->pos.dense.ptr);
        free(r->vel.sparse.ptr); free(r->vel.ents.ptr); free(r->vel.dense.ptr);
        memset(r, 0, sizeof *r);
        h_seed(seed);
        uint32_t n0 = 1 + h_rand() % 60;
        for (uint32_t i = 0; i < n0; i++) {
            uint32_t e = reg_create(r);
            if (h_rand() % 4) reg_add_pos(r, e, (V2){0, 0});
            if (h_rand() % 3) reg_add_vel(r, e, (V2){0, 0});
        }
        static uint8_t expected[8192], visited[8192];
        memset(expected, 0, sizeof expected); memset(visited, 0, sizeof visited);
        for (int64_t i = 0; i < r->glen; i++) expected[r->pos.ents.ptr[i]] = 1;
        int64_t saved = r->cursor, k = r->glen;
        while (k-- > 0) {
            uint32_t e = r->pos.ents.ptr[k];
            CHECK(expected[e] == 1 && !visited[e], "check: visited twice, or an entity that joined during the loop was visited");
            visited[e] = 1;
            uint32_t act = h_rand() % 6;
            if (act >= 4) continue;
            r->cursor = k;                                 /* published before calls that read it */
            if (act == 0) { reg_destroy(r, e); check_pools(r); }
            uint32_t extra = h_rand() % 4;
            for (uint32_t x = 0; x < extra; x++) {
                uint32_t v = h_rand() % r->next_id, op = h_rand() % 4;
                if (op == 0 && (pool_has(&r->pos, v) || pool_has(&r->vel, v))) {          /* destroy another */
                    if (in_group(r, v) && !visited[v]) expected[v] = 0;   /* gone before its turn */
                    reg_destroy(r, v);
                } else if (op == 1 && pool_has(&r->vel, v)) {                               /* take Velocity away */
                    if (in_group(r, v) && !visited[v]) expected[v] = 0;   /* left before its turn */
                    reg_remove_vel(r, v);
                } else if (op == 2 && pool_has(&r->pos, v) && !pool_has(&r->vel, v)) {     /* it joins the group */
                    reg_add_vel(r, v, (V2){0, 0});
                } else if (op == 3) {                                                       /* a new entity joins */
                    uint32_t n = reg_create(r); reg_add_pos(r, n, (V2){0, 0}); reg_add_vel(r, n, (V2){0, 0});
                }
                check_pools(r);
            }
            k = r->cursor;                                 /* read back */
            CHECK(k <= r->glen, "check: cursor past the group");
        }
        r->cursor = saved;
        for (uint32_t e = 0; e < r->next_id && e < 8192; e++)
            CHECK(!(expected[e] && !visited[e]), "check: an expected entity was not visited");
    }
    printf("cursor-check ok\n");
    return 0;
}
#else
int main(void) {
    h_start();
    static Registry reg;                         /* zeroed: empty pools, glen 0, cursor 0 */
    Registry *r = &reg;
    for (uint32_t i = 0; i < N; i++) {
        uint32_t e = reg_create(r);
        reg_add_pos(r, e, (V2){(float)(i % 1000), (float)(i / 1000)});
        if (i % 2 == 0) reg_add_vel(r, e, (V2){(float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f});
    }
    for (uint32_t s = 0; s < 100; s++) {
        int64_t saved = r->cursor;                /* loops nest: keep the outer loop's cursor */
        int64_t k = r->glen;
        uint32_t *ents = r->pos.ents.ptr; V2 *pd = r->pos.dense.ptr, *vd = r->vel.dense.ptr;
        CHECK(k <= r->pos.ents.len && k <= r->pos.dense.len && k <= r->vel.dense.len, "index out of bounds");
        while (k-- > 0) {
            uint32_t e = ents[k];
            V2 *pos = &pd[k], *vel = &vd[k];      /* the names pos and vel: slot k of each pool */
            pos->x += vel->x;
            pos->y += vel->y;
            if (e % 100u == s) {                  /* pos and vel are not used after this line */
                r->cursor = k;                    /* destroy's summary reads the cursor */
                reg_destroy(r, e);
                uint32_t n = reg_create(r);
                reg_add_pos(r, n, (V2){0.0f, 0.0f});
                reg_add_vel(r, n, (V2){1.0f, 1.0f});
                k = r->cursor;                    /* read back: a leave may have moved it */
                ents = r->pos.ents.ptr; pd = r->pos.dense.ptr; vd = r->vel.dense.ptr;
                CHECK(k <= r->pos.ents.len && k <= r->pos.dense.len && k <= r->vel.dense.len, "index out of bounds");
            }
        }
        r->cursor = saved;
    }
    uint64_t sum = 0;
    for (int64_t j = 0; j < r->pos.dense.len; j++)
        sum += (uint64_t)((int64_t)(r->pos.dense.ptr[j].x * 4.0f) + (int64_t)(r->pos.dense.ptr[j].y * 4.0f));
    h_report("combined-w1", sum + (uint64_t)r->pos.dense.len);
    return 0;
}
#endif
