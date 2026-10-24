/* ecs_view_cost.c: what problem 3's fix costs (tasks/96/redesign/phase3b-combined-attack.md): a view
   over a pool that a group owns, whose body may change that group's members, walks a copy of the
   pool's entity ids. W1's entities and W1's body, written as a view over Position (every entity with
   a Position: 1,000,000), in EnTT's owning-group layout (combined/w1.c's library):

       for s in range(100):
           for e, pos in reg.view(Position):
               pos.x += 0.5
               if e.id % 100 == s:
                   reg.destroy(e)
                   n = reg.create()
                   reg.add(n, Position(0, 0)); reg.add(n, Velocity(1, 1))   # n joins the group

   Models (-DMODEL=n), the same visits and the same checksum for 0 and 1:
     0  plain C as a careful C programmer writes it for EnTT: walk the pool backwards, destroy the
        current entity in place (allowed), and keep the spawns for after the loop (EnTT does not allow
        adding owned components during the loop); no checks
     1  the fix as the model's compiler would emit it: copy the pool's ids when the loop starts, walk
        the copy backwards, find each one's slot through `sparse` (bounds checks the compiler cannot
        prove), spawn in place
     2  for reference only, NOT the same work: the cursor walk as written, spawning in place (problem 3:
        it visits some spawned entities and skips others), no checks
     3  the cheaper fix for a body that only creates entities and destroys the current one, as the
        model's compiler would emit it: the cursor walk; a new entity gets its components at once but
        its move into the group waits in a pending list until the loop ends (a group loop nested in
        the view would also walk that list; none here); bounds checks as model 1
     4  plain C with model 1's meaning: the copy of the ids, no checks
   Build: clang -O2 -DMODEL=n ecs_view_cost.c -o ecs_view_cost_n.exe */
#include "../../redesign/bench/harness.h"
#include <string.h>
#ifndef MODEL
#define MODEL 0
#endif
#define N 1000000u
#define NONE 0xFFFFFFFFu
static __declspec(noreturn) void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#if MODEL == 1 || MODEL == 3
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)
#else
#define CHECK(c, m) ((void)0)
#endif

typedef struct { float x, y; } V2;
typedef struct { uint32_t *ptr; int64_t len, cap; } ArrU32;
typedef struct { V2 *ptr; int64_t len, cap; } ArrV2;
typedef struct { ArrU32 sparse, ents; ArrV2 dense; } Pool;
typedef struct { Pool pos, vel; int64_t glen; uint32_t next_id; } Registry;

static void push_u32(ArrU32 *a, uint32_t v) { if (a->len == a->cap) { a->cap = a->cap ? a->cap * 2 : 4; a->ptr = realloc(a->ptr, (size_t)a->cap * 4); if (!a->ptr) panic("oom"); } a->ptr[a->len++] = v; }
static void push_v2(ArrV2 *a, V2 v) { if (a->len == a->cap) { a->cap = a->cap ? a->cap * 2 : 4; a->ptr = realloc(a->ptr, (size_t)a->cap * sizeof(V2)); if (!a->ptr) panic("oom"); } a->ptr[a->len++] = v; }
static void sparse_cover(ArrU32 *s, uint32_t id) {
    if ((int64_t)id < s->len) return;
    int64_t nc = s->cap ? s->cap : 1024; while (nc <= (int64_t)id) nc *= 2;
    s->ptr = realloc(s->ptr, (size_t)nc * 4); if (!s->ptr) panic("oom");
    for (int64_t i = s->len; i < nc; i++) s->ptr[i] = NONE;
    s->len = s->cap = nc;
}
static inline int pool_has(const Pool *p, uint32_t id) { return (int64_t)id < p->sparse.len && p->sparse.ptr[id] != NONE; }
static void pool_add(Pool *p, uint32_t id, V2 v) { sparse_cover(&p->sparse, id); p->sparse.ptr[id] = (uint32_t)p->ents.len; push_u32(&p->ents, id); push_v2(&p->dense, v); }
static void pool_swap(Pool *p, int64_t i, int64_t j) {
    if (i == j) return;
    CHECK(i >= 0 && j >= 0 && i < p->ents.len && j < p->ents.len, "index out of bounds");
    uint32_t a = p->ents.ptr[i], b = p->ents.ptr[j]; V2 t = p->dense.ptr[i];
    p->ents.ptr[i] = b; p->ents.ptr[j] = a; p->dense.ptr[i] = p->dense.ptr[j]; p->dense.ptr[j] = t;
    p->sparse.ptr[b] = (uint32_t)i; p->sparse.ptr[a] = (uint32_t)j;
}
static void pool_del(Pool *p, uint32_t id) {
    int64_t i = p->sparse.ptr[id], last = p->ents.len - 1;
    if (i != last) { uint32_t moved = p->ents.ptr[last]; p->ents.ptr[i] = moved; p->dense.ptr[i] = p->dense.ptr[last]; p->sparse.ptr[moved] = (uint32_t)i; }
    p->sparse.ptr[id] = NONE; p->ents.len--; p->dense.len--;
}
static void group_enter(Registry *r, uint32_t id) { pool_swap(&r->pos, r->pos.sparse.ptr[id], r->glen); pool_swap(&r->vel, r->vel.sparse.ptr[id], r->glen); r->glen++; }
static void group_leave(Registry *r, uint32_t id) { int64_t j = r->pos.sparse.ptr[id]; r->glen--; pool_swap(&r->pos, j, r->glen); pool_swap(&r->vel, j, r->glen); }
static void add_pos(Registry *r, uint32_t id, V2 v) { pool_add(&r->pos, id, v); if (pool_has(&r->vel, id)) group_enter(r, id); }
static void add_vel(Registry *r, uint32_t id, V2 v) { pool_add(&r->vel, id, v); if (pool_has(&r->pos, id)) group_enter(r, id); }
static void destroy(Registry *r, uint32_t id) {
    int hp = pool_has(&r->pos, id), hv = pool_has(&r->vel, id);
    if (hp && hv && (int64_t)r->pos.sparse.ptr[id] < r->glen) group_leave(r, id);
    if (hp) pool_del(&r->pos, id);
    if (hv) pool_del(&r->vel, id);
}
static void spawn(Registry *r) { uint32_t n = r->next_id++; add_pos(r, n, (V2){0, 0}); add_vel(r, n, (V2){1, 1}); }

int main(void) {
    h_start();
    static Registry reg; Registry *r = &reg;
    for (uint32_t i = 0; i < N; i++) {
        uint32_t e = r->next_id++;
        add_pos(r, e, (V2){(float)(i % 1000), (float)(i / 1000)});
        if (i % 2 == 0) add_vel(r, e, (V2){(float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f});
    }
#if MODEL == 3
    uint32_t *pend = malloc((size_t)N * 4); int64_t pend_cap = N;
    if (!pend) panic("oom");
#endif
#if MODEL == 1 || MODEL == 4
    uint32_t *copy = malloc((size_t)N * 2 * 4); int64_t copy_cap = (int64_t)N * 2;
    if (!copy) panic("oom");
#endif
    for (uint32_t s = 0; s < 100; s++) {
#if MODEL == 0
        uint32_t pending = 0;
        for (int64_t k = r->pos.ents.len; k-- > 0;) {
            uint32_t e = r->pos.ents.ptr[k];
            r->pos.dense.ptr[k].x += 0.5f;
            if (e % 100u == s) { destroy(r, e); pending++; }
        }
        while (pending--) spawn(r);
#elif MODEL == 3
        int64_t np = 0;
        int64_t k = r->pos.ents.len;
        uint32_t *ents = r->pos.ents.ptr; V2 *pd = r->pos.dense.ptr;
        while (k-- > 0) {
            uint32_t e = ents[k];
            pd[k].x += 0.5f;
            if (e % 100u == s) {
                destroy(r, e);                                      /* the current entity: allowed */
                uint32_t n = r->next_id++;
                pool_add(&r->pos, n, (V2){0, 0}); pool_add(&r->vel, n, (V2){1, 1});   /* appended: past the cursor */
                if (np == pend_cap) { pend_cap *= 2; pend = realloc(pend, (size_t)pend_cap * 4); if (!pend) panic("oom"); }
                pend[np++] = n;                                     /* joins the group when the loop ends */
                ents = r->pos.ents.ptr; pd = r->pos.dense.ptr;      /* read again after the change */
                CHECK(k <= r->pos.ents.len && k <= r->pos.dense.len, "index out of bounds");
            }
        }
        for (int64_t j = 0; j < np; j++)
            if (pool_has(&r->pos, pend[j]) && pool_has(&r->vel, pend[j]) && (int64_t)r->pos.sparse.ptr[pend[j]] >= r->glen) group_enter(r, pend[j]);
#elif MODEL == 1 || MODEL == 4
        int64_t n = r->pos.ents.len;
        if (n > copy_cap) { copy_cap = n * 2; copy = realloc(copy, (size_t)copy_cap * 4); if (!copy) panic("oom"); }
        memcpy(copy, r->pos.ents.ptr, (size_t)n * 4);
        for (int64_t k = n; k-- > 0;) {
            uint32_t e = copy[k];
            if (!pool_has(&r->pos, e)) continue;                  /* gone before its turn */
            uint32_t slot = r->pos.sparse.ptr[e];
            CHECK((int64_t)slot < r->pos.dense.len, "index out of bounds");
            r->pos.dense.ptr[slot].x += 0.5f;
            if (e % 100u == s) { destroy(r, e); spawn(r); }
        }
#else
        for (int64_t k = r->pos.ents.len; k-- > 0;) {
            uint32_t e = r->pos.ents.ptr[k];
            r->pos.dense.ptr[k].x += 0.5f;
            if (e % 100u == s) { destroy(r, e); spawn(r); }
        }
#endif
    }
    uint64_t sum = 0;
    for (int64_t j = 0; j < r->pos.dense.len; j++)
        sum += (uint64_t)((int64_t)(r->pos.dense.ptr[j].x * 4.0f) + (int64_t)(r->pos.dense.ptr[j].y * 4.0f));
    static const char *names[] = {"view-owned-plainC-deferred-spawns", "view-owned-copy-of-ids", "view-owned-cursor-as-written-NOT-same-work",
                                  "view-owned-pending-group-entry", "view-owned-plainC-copy-of-ids"};
    h_report(names[MODEL], sum + (uint64_t)r->pos.dense.len);
    return 0;
}
