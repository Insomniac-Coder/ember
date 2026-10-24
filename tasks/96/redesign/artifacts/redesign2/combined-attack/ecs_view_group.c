/* ecs_view_group.c: the combined model's ECS library (phase3b-combined.md 6.1) has two loops, `view`
   (a pool walked backwards with the pool's own cursor; Pool.leave applies the cursor rule) and
   `group` (the owning group's range walked with the group's cursor; the group's leave applies the
   rule). Problem 3 of tasks/96/redesign/phase3b-combined-attack.md: what happens when a `view` walks a
   pool that a group owns, and the body changes who is in the group.

   The library below is combined/w1.c's (pool_add, pool_swap, group_enter, group_leave with the group's
   cursor rule, reg_destroy, reg_remove_vel), plus the pieces of section 6.1's Ember text that w1.c
   does not need: the pool's own cursor, Pool.leave's cursor rule, and the view loop. A third
   component, Health, is owned by no group.

   The Python a game writes (the registry of dicts in combined/python/w1_dicts.py, whose view walks a
   copy of the keys):
       for e, pos in reg.view(Position):              # every entity with a Position
           pos.x += 1
           if hit(e): reg.add(other, Velocity(...))   # an entity that had only a Position starts moving
           if stopped(e2): reg.remove(e2, Velocity)   # another one stops
   Check, 5,000 random loops per view kind: every entity that has the view's components when the loop
   starts, and still has them at its turn, is visited exactly once; no entity created during the loop
   is visited; the pools and the group stay consistent.
     as written:  clang -O2 ecs_view_group.c -o ecs_view_group_as_written.exe
     saved:       clang -O2 -DSAVED ecs_view_group.c -o ecs_view_group_saved.exe
   -DSPAWN_ONLY (with either): the body only destroys and creates entities with both components, the
   changes W1's own loop body makes.
   SAVED: the view walks a pool that no group owns when it has one (Health); when every pool of the
   view is owned by a group and the body may add or remove a component that group owns (the body's
   summary says so), the view walks a copy of the walked pool's entity ids taken when it starts and
   looks each one up (Python's `for e in list(a)`); otherwise the cursor walk as written. */
#include "../../redesign/bench/harness.h"
#include <string.h>

#define NONE 0xFFFFFFFFu
static __declspec(noreturn) void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)

typedef struct { float x, y; } V2;
typedef struct { uint32_t *ptr; int64_t len, cap; } ArrU32;
typedef struct { V2 *ptr; int64_t len, cap; } ArrV2;
typedef struct { ArrU32 sparse, ents; ArrV2 dense; int64_t cursor; int walking; } Pool;   /* + the view cursor */
typedef struct { Pool pos, vel, hp; int64_t glen, cursor; uint32_t next_id; } Registry;

static void push_u32(ArrU32 *a, uint32_t v) { if (a->len == a->cap) { a->cap = a->cap ? a->cap * 2 : 4; a->ptr = realloc(a->ptr, (size_t)a->cap * 4); } a->ptr[a->len++] = v; }
static void push_v2(ArrV2 *a, V2 v) { if (a->len == a->cap) { a->cap = a->cap ? a->cap * 2 : 4; a->ptr = realloc(a->ptr, (size_t)a->cap * sizeof(V2)); } a->ptr[a->len++] = v; }
static void sparse_cover(ArrU32 *s, uint32_t id) {
    if ((int64_t)id < s->len) return;
    int64_t nc = s->cap ? s->cap : 1024; while (nc <= (int64_t)id) nc *= 2;
    s->ptr = realloc(s->ptr, (size_t)nc * 4);
    for (int64_t i = s->len; i < nc; i++) s->ptr[i] = NONE;
    s->len = s->cap = nc;
}
static inline int pool_has(const Pool *p, uint32_t id) { return (int64_t)id < p->sparse.len && p->sparse.ptr[id] != NONE; }
static void pool_add(Pool *p, uint32_t id, V2 v) { sparse_cover(&p->sparse, id); p->sparse.ptr[id] = (uint32_t)p->ents.len; push_u32(&p->ents, id); push_v2(&p->dense, v); }
static void pool_swap(Pool *p, int64_t i, int64_t j) {
    if (i == j) return;
    uint32_t a = p->ents.ptr[i], b = p->ents.ptr[j]; V2 t = p->dense.ptr[i];
    p->ents.ptr[i] = b; p->ents.ptr[j] = a; p->dense.ptr[i] = p->dense.ptr[j]; p->dense.ptr[j] = t;
    p->sparse.ptr[b] = (uint32_t)i; p->sparse.ptr[a] = (uint32_t)j;
}
/* Pool.leave of section 6.1: "every removal from the walked range comes through here" */
static void pool_leave(Pool *p, uint32_t id) {
    int64_t i = p->sparse.ptr[id];
    if (p->walking && i < p->cursor) { p->cursor -= 1; pool_swap(p, i, p->cursor); i = p->cursor; }
    pool_swap(p, i, p->ents.len - 1);
    p->sparse.ptr[id] = NONE; p->ents.len--; p->dense.len--;
}
/* combined/w1.c, unchanged */
static void group_enter(Registry *r, uint32_t id) { pool_swap(&r->pos, r->pos.sparse.ptr[id], r->glen); pool_swap(&r->vel, r->vel.sparse.ptr[id], r->glen); r->glen++; }
static void group_leave(Registry *r, uint32_t id) {
    int64_t j = r->pos.sparse.ptr[id];
    if (j < r->cursor) { int64_t c = --r->cursor; pool_swap(&r->pos, j, c); pool_swap(&r->vel, j, c); j = c; }
    r->glen--;
    pool_swap(&r->pos, j, r->glen); pool_swap(&r->vel, j, r->glen);
}
static inline int in_group(const Registry *r, uint32_t id) { return pool_has(&r->pos, id) && pool_has(&r->vel, id) && (int64_t)r->pos.sparse.ptr[id] < r->glen; }
static void add_pos(Registry *r, uint32_t id) { pool_add(&r->pos, id, (V2){0, 0}); if (pool_has(&r->vel, id)) group_enter(r, id); }
static void add_vel(Registry *r, uint32_t id) { pool_add(&r->vel, id, (V2){0, 0}); if (pool_has(&r->pos, id)) group_enter(r, id); }
static void add_hp(Registry *r, uint32_t id) { pool_add(&r->hp, id, (V2){0, 0}); }
static void destroy(Registry *r, uint32_t id) {
    if (in_group(r, id)) group_leave(r, id);
    if (pool_has(&r->pos, id)) pool_leave(&r->pos, id);
    if (pool_has(&r->vel, id)) pool_leave(&r->vel, id);
    if (pool_has(&r->hp, id)) pool_leave(&r->hp, id);
}
static void remove_vel(Registry *r, uint32_t id) { if (!pool_has(&r->vel, id)) return; if (in_group(r, id)) group_leave(r, id); pool_leave(&r->vel, id); }

static void check_pools(Registry *r) {
    Pool *ps[3] = {&r->pos, &r->vel, &r->hp};
    for (int q = 0; q < 3; q++) for (int64_t i = 0; i < ps[q]->ents.len; i++) CHECK(ps[q]->sparse.ptr[ps[q]->ents.ptr[i]] == (uint32_t)i, "check: sparse");
    for (int64_t i = 0; i < r->pos.ents.len; i++) {
        uint32_t e = r->pos.ents.ptr[i]; int both = pool_has(&r->vel, e);
        CHECK((i < r->glen) == both, "check: group membership");
        if (both) CHECK(r->vel.ents.ptr[i] == e, "check: group order");
    }
}

static uint8_t expected[8192], visited[8192];
static int64_t failures, loops;
#ifdef SAVED
static uint32_t snap[8192];
#endif

/* the body: random changes to OTHER entities' components, as a game's systems make them */
static void body(Registry *r, uint32_t e, int with_hp) {
    (void)e;
    uint32_t extra = h_rand() % 4;
    for (uint32_t x = 0; x < extra; x++) {
        uint32_t v = h_rand() % r->next_id, op = h_rand() % 5;
#ifdef SPAWN_ONLY   /* control: only W1's own changes: destroy, and create an entity with both components */
        if (op == 1 || op == 2 || op == 4) op = 3;
#endif
        int in_view = pool_has(&r->pos, v) && (!with_hp || pool_has(&r->hp, v));
        if (op == 0 && pool_has(&r->pos, v)) { if (in_view && !visited[v]) expected[v] = 0; destroy(r, v); }
        else if (op == 1 && pool_has(&r->vel, v)) remove_vel(r, v);                          /* stops moving */
        else if (op == 2 && pool_has(&r->pos, v) && !pool_has(&r->vel, v)) add_vel(r, v);   /* starts moving */
        else if (op == 3) { uint32_t n = r->next_id++; add_pos(r, n); add_vel(r, n); if (with_hp) add_hp(r, n); }
        else if (op == 4) { uint32_t n = r->next_id++; add_pos(r, n); if (with_hp) add_hp(r, n); }
        check_pools(r);
    }
}
static void visit(uint32_t e) {
    if (!(expected[e] == 1 && !visited[e])) failures++;
    visited[e] = 1;
}

/* view[Position] (with_hp 0) or view[Position, Health] (with_hp 1) */
static void run_view(Registry *r, int with_hp) {
    memset(expected, 0, sizeof expected); memset(visited, 0, sizeof visited);
    for (int64_t i = 0; i < r->pos.ents.len; i++) { uint32_t e = r->pos.ents.ptr[i]; if (!with_hp || pool_has(&r->hp, e)) expected[e] = 1; }
#ifndef SAVED
    Pool *a = &r->pos; Pool *b = with_hp ? &r->hp : NULL;            /* as written: the walked pool is Position */
    int64_t saved = a->cursor; int sw = a->walking; a->walking = 1; a->cursor = a->ents.len;
    while (a->cursor > 0) {
        a->cursor -= 1;
        uint32_t e = a->ents.ptr[a->cursor];
        if (b && !pool_has(b, e)) continue;
        visit(e); body(r, e, with_hp);
    }
    a->cursor = saved; a->walking = sw;
#else
    if (with_hp) {                                                   /* a pool no group owns: the cursor walk */
        Pool *a = &r->hp;
        int64_t saved = a->cursor; int sw = a->walking; a->walking = 1; a->cursor = a->ents.len;
        while (a->cursor > 0) {
            a->cursor -= 1;
            uint32_t e = a->ents.ptr[a->cursor];
            if (!pool_has(&r->pos, e)) continue;
            visit(e); body(r, e, with_hp);
        }
        a->cursor = saved; a->walking = sw;
    } else {                                                         /* every pool owned, the body changes the group: a copy */
        int64_t n = r->pos.ents.len;
        memcpy(snap, r->pos.ents.ptr, (size_t)n * 4);
        for (int64_t k = n; k-- > 0;) {
            uint32_t e = snap[k];
            if (!pool_has(&r->pos, e)) continue;                     /* destroyed before its turn */
            visit(e); body(r, e, with_hp);
        }
    }
#endif
    for (uint32_t e = 0; e < r->next_id && e < 8192; e++)
        if (expected[e] && !visited[e] && pool_has(&r->pos, e) && (!with_hp || pool_has(&r->hp, e))) failures++;   /* skipped */
    loops++;
}

int main(void) {
    h_start();
    int64_t fail_by_kind[2] = {0, 0};
    for (int with_hp = 0; with_hp < 2; with_hp++) {
        failures = 0;
        for (uint64_t seed = 1; seed <= 5000; seed++) {
            static Registry reg; Registry *r = &reg;
            Pool *ps[3] = {&r->pos, &r->vel, &r->hp};
            for (int q = 0; q < 3; q++) { free(ps[q]->sparse.ptr); free(ps[q]->ents.ptr); free(ps[q]->dense.ptr); }
            memset(r, 0, sizeof *r);
            h_seed(seed);
            uint32_t n0 = 1 + h_rand() % 60;
            for (uint32_t i = 0; i < n0; i++) {
                uint32_t e = r->next_id++;
                if (h_rand() % 4) add_pos(r, e);
                if (h_rand() % 3) add_vel(r, e);
                if (h_rand() % 2) add_hp(r, e);
            }
            int64_t before = failures;
            run_view(r, with_hp);
            if (failures != before && fail_by_kind[with_hp] == 0) fail_by_kind[with_hp] = (int64_t)seed;
        }
        printf("%s: view[%s]: 5000 loops, %lld visits wrong (an entity visited twice, visited after joining, or skipped); first failing seed %lld\n",
#ifdef SAVED
               "saved",
#else
               "as written",
#endif
               with_hp ? "Position, Health" : "Position", (long long)failures, (long long)fail_by_kind[with_hp]);
    }
    return 0;
}
