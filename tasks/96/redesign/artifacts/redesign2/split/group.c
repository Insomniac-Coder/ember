/* Copied from salvage/plain/w1_group.c; -DLOCAL added (phase3b-measurements.md).
   Plain C, W1 (ECS) in EnTT's owning-group layout. No safety checks.
   One sparse set per component (sparse[id] -> slot, ents[slot] -> id, data[slot]). The group
   <Position, Velocity> owns both pools: the entities that have both sit in slots [0, glen) of
   BOTH pools, in the same order, so the step loop reads data[k] of both pools straight through.
   The loop runs backwards (EnTT's order): a despawn swaps the last group member (already
   visited) into slot k, and a spawn joins the group at slot glen-1, past the cursor. */
#include "harness.h"
#define N 1000000
#define NONE 0xFFFFFFFFu
typedef struct { float x, y; } V2;
typedef struct { uint32_t *sparse; size_t scap; uint32_t *ents; V2 *data; size_t n, cap; } Pool;
static Pool pos, vel;
static size_t glen;

static void grow_sparse(Pool *p, uint32_t id) {
    if (id < p->scap) return;
    size_t nc = p->scap ? p->scap : 1024;
    while (nc <= id) nc *= 2;
    p->sparse = realloc(p->sparse, nc * 4);
    for (size_t i = p->scap; i < nc; i++) p->sparse[i] = NONE;
    p->scap = nc;
}
static inline int has(const Pool *p, uint32_t id) { return id < p->scap && p->sparse[id] != NONE; }
static void add(Pool *p, uint32_t id, V2 v) {
    grow_sparse(p, id);
    if (p->n == p->cap) {
        p->cap = p->cap ? p->cap * 2 : 4;
        p->ents = realloc(p->ents, p->cap * 4); p->data = realloc(p->data, p->cap * sizeof(V2));
    }
    p->sparse[id] = (uint32_t)p->n; p->ents[p->n] = id; p->data[p->n] = v; p->n++;
}
static void swap_slots(Pool *p, size_t i, size_t j) {
    if (i == j) return;
    uint32_t a = p->ents[i], b = p->ents[j]; V2 t = p->data[i];
    p->ents[i] = b; p->ents[j] = a; p->data[i] = p->data[j]; p->data[j] = t;
    p->sparse[b] = (uint32_t)i; p->sparse[a] = (uint32_t)j;
}
static void del(Pool *p, uint32_t id) { /* swap-and-pop */
    uint32_t i = p->sparse[id], last = p->ents[--p->n];
    p->ents[i] = last; p->data[i] = p->data[p->n]; p->sparse[last] = i; p->sparse[id] = NONE;
}
static void group_enter(uint32_t id) { swap_slots(&pos, pos.sparse[id], glen); swap_slots(&vel, vel.sparse[id], glen); glen++; }
static void group_leave(uint32_t id) { glen--; swap_slots(&pos, pos.sparse[id], glen); swap_slots(&vel, vel.sparse[id], glen); }
static void emplace_pos(uint32_t id, V2 v) { add(&pos, id, v); if (has(&vel, id)) group_enter(id); }
static void emplace_vel(uint32_t id, V2 v) { add(&vel, id, v); if (has(&pos, id)) group_enter(id); }
static void destroy(uint32_t id) {
    int hp = has(&pos, id), hv = has(&vel, id);
    if (hp && hv && pos.sparse[id] < glen) group_leave(id);
    if (hp) del(&pos, id);
    if (hv) del(&vel, id);
}

int main(void) {
    h_start();
    uint32_t next = 0;
    for (uint32_t i = 0; i < N; i++) {
        uint32_t e = next++;
        emplace_pos(e, (V2){(float)(i % 1000), (float)(i / 1000)});
        if (i % 2 == 0) emplace_vel(e, (V2){(float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f});
    }
    { LARGE_INTEGER t, f; QueryPerformanceCounter(&t); QueryPerformanceFrequency(&f); printf("setup_ms=%.3f ", (double)(t.QuadPart - h_t0.QuadPart) * 1000.0 / (double)f.QuadPart); }
    for (uint32_t s = 0; s < 100; s++) {
#ifdef LOCAL /* the array pointers kept in locals, fetched again only after the rare despawn/spawn */
        uint32_t *ents = pos.ents; V2 *pd = pos.data, *vd = vel.data;
        for (size_t k = glen; k-- > 0;) {
            uint32_t e = ents[k];
            V2 *p = &pd[k], *v = &vd[k];
            p->x += v->x; p->y += v->y;
            if (e % 100 == s) {
                destroy(e);
                uint32_t n = next++;
                emplace_pos(n, (V2){0, 0});
                emplace_vel(n, (V2){1, 1});
                ents = pos.ents; pd = pos.data; vd = vel.data;
            }
        }
#else
        for (size_t k = glen; k-- > 0;) {
            uint32_t e = pos.ents[k];
            V2 *p = &pos.data[k], *v = &vel.data[k];
            p->x += v->x; p->y += v->y;
            if (e % 100 == s) {
                destroy(e);
                uint32_t n = next++;
                emplace_pos(n, (V2){0, 0});
                emplace_vel(n, (V2){1, 1});
            }
        }
#endif
    }
    uint64_t sum = 0;
    for (size_t j = 0; j < pos.n; j++)
        sum += (uint64_t)((int64_t)(pos.data[j].x * 4.0f) + (int64_t)(pos.data[j].y * 4.0f));
    h_report(
#ifdef LOCAL
        "plain-w1-group-local",
#else
        "plain-w1-group",
#endif
        sum + pos.n);
    return 0;
}
