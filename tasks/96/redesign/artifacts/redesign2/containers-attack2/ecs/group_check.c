/* ECS attack in the layout the owner chose (EnTT's full-owning group): does the containers group's
   cursor rule ("destroying an entity the loop has not reached yet moves the last unreached one into
   its slot, then removes that slot as a visited one") keep "every entity once" when the loop body also
   REMOVES ONE COMPONENT from entities (the entity leaves the group but lives on) and ADDS the missing
   component to entities that had only one (the entity joins the group during the loop)?
   The library below (pools, group, create, destroy) is copied unchanged from the combined model,
   target/redesign2/combined/w1.c (lines between its harness include and its CURSOR_CHECK block).
   remove_vel() is the library function the containers answer needs as well: built two ways,
     -DREMOVE_AS_DESTROY_WITHOUT_CURSOR : it leaves the group with a plain swap (no cursor rule)
     (default)                          : it leaves the group with destroy()'s cursor rule
   Random loops (5,000 seeds); after every change the pools must be consistent; every entity that is in
   the group when the loop starts and still in it at its turn is visited exactly once; an entity that
   joins during the loop is not visited. Prints "group-check ok". */
#include "../../../redesign/bench/harness.h"
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
static void reg_add_pos(Registry *r, uint32_t id, V2 v) { pool_add(&r->pos, id, v); if (pool_has(&r->vel, id)) group_enter(r, id); }
static void reg_add_vel(Registry *r, uint32_t id, V2 v) { pool_add(&r->vel, id, v); if (pool_has(&r->pos, id)) group_enter(r, id); }
static uint32_t reg_create(Registry *r) { CHECK(r->next_id != NONE, "entity ids exhausted"); return r->next_id++; }
static void reg_destroy(Registry *r, uint32_t id) {
    int hp = pool_has(&r->pos, id), hv = pool_has(&r->vel, id);
    if (hp && hv) {
        int64_t j = r->pos.sparse.ptr[id];
        if (j < r->glen) {                                /* in the group: leave it first */
            if (__builtin_expect(j < r->cursor, 0)) {     /* a running loop has not reached it yet */
                int64_t c = --r->cursor;                  /* the last slot not yet visited */
                pool_swap(&r->pos, j, c); pool_swap(&r->vel, j, c);
                j = c;                                    /* now leave from c, a visited slot */
            }
            r->glen--;
            pool_swap(&r->pos, j, r->glen); pool_swap(&r->vel, j, r->glen);
        }
    }
    if (hp) pool_del(&r->pos, id);
    if (hv) pool_del(&r->vel, id);
}

static void group_leave(Registry *r, uint32_t id) {     /* id is in both pools */
    int64_t j = r->pos.sparse.ptr[id];
    if (j >= r->glen) return;
#ifndef REMOVE_AS_DESTROY_WITHOUT_CURSOR
    if (j < r->cursor) {                                  /* the same rule as reg_destroy */
        int64_t c = --r->cursor;
        pool_swap(&r->pos, j, c); pool_swap(&r->vel, j, c);
        j = c;
    }
#endif
    r->glen--;
    pool_swap(&r->pos, j, r->glen); pool_swap(&r->vel, j, r->glen);
}
static void reg_remove_vel(Registry *r, uint32_t id) {   /* reg.remove[Velocity](id) */
    if (!pool_has(&r->vel, id)) return;
    if (pool_has(&r->pos, id)) group_leave(r, id);
    pool_del(&r->vel, id);
}
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
#define MAXE 8192
int main(void) {
    h_start();
    static Registry reg; Registry *r = &reg;
    static uint8_t expected[MAXE], visited[MAXE], joined[MAXE];
    for (uint64_t seed = 1; seed <= 5000; seed++) {
        free(r->pos.sparse.ptr); free(r->pos.ents.ptr); free(r->pos.dense.ptr);
        free(r->vel.sparse.ptr); free(r->vel.ents.ptr); free(r->vel.dense.ptr);
        memset(r, 0, sizeof *r);
        h_seed(seed);
        uint32_t n0 = 1 + h_rand() % 80;
        for (uint32_t i = 0; i < n0; i++) {
            uint32_t e = reg_create(r);
            if (h_rand() % 4) reg_add_pos(r, e, (V2){0, 0});
            if (h_rand() % 3) reg_add_vel(r, e, (V2){0, 0});
        }
        memset(expected, 0, sizeof expected); memset(visited, 0, sizeof visited); memset(joined, 0, sizeof joined);
        for (int64_t i = 0; i < r->glen; i++) expected[r->pos.ents.ptr[i]] = 1;
        int64_t saved = r->cursor, k = r->glen;
        while (k-- > 0) {
            uint32_t e = r->pos.ents.ptr[k];
            if (joined[e]) { printf("seed %llu: entity %u joined during the loop and was visited\n", (unsigned long long)seed, e); return 1; }
            if (!expected[e] || visited[e]) { printf("seed %llu: entity %u visited twice or not expected\n", (unsigned long long)seed, e); return 1; }
            visited[e] = 1;
            if (h_rand() % 2) continue;
            r->cursor = k;                                 /* published before calls that may change the group */
            uint32_t ops = 1 + h_rand() % 4;
            for (uint32_t q = 0; q < ops; q++) {
                uint32_t act = h_rand() % 5, v = h_rand() % r->next_id;
                if (act == 0) v = e;                       /* the current entity */
                if (act <= 1) {                            /* reg.destroy(v) */
                    if (!pool_has(&r->pos, v) && !pool_has(&r->vel, v)) continue;
                    if (!visited[v]) expected[v] = 0;
                    reg_destroy(r, v);
                } else if (act == 2) {                     /* reg.remove[Velocity](v): leaves the group, lives on */
                    if (!pool_has(&r->vel, v)) continue;
                    if (!visited[v]) expected[v] = 0;
                    reg_remove_vel(r, v);
                } else if (act == 3) {                     /* reg.add(v, Velocity(...)): may join the group */
                    if (pool_has(&r->vel, v) || !pool_has(&r->pos, v)) continue;
                    reg_add_vel(r, v, (V2){0, 0});
                    if (!visited[v]) joined[v] = 1;
                } else {                                   /* create one with both: joins the group */
                    uint32_t n = reg_create(r);
                    if (n >= MAXE) continue;
                    reg_add_pos(r, n, (V2){0, 0}); reg_add_vel(r, n, (V2){0, 0});
                    joined[n] = 1;
                }
                check_pools(r);
            }
            k = r->cursor;
            CHECK(k <= r->glen, "check: cursor past the group");
        }
        r->cursor = saved;
        for (uint32_t x = 0; x < r->next_id && x < MAXE; x++)
            if (expected[x] != visited[x]) { printf("seed %llu: entity %u expected %d visited %d\n", (unsigned long long)seed, x, expected[x], visited[x]); return 1; }
    }
    printf("group-check ok\n");
    return 0;
}
