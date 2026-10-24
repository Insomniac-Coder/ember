/* retarget.c: tasks that re-target their own players (probes/y22): `p.target = pick(enemies)` in a parallel
 * loop. The old target is released and the new one retained; both are objects the parent owns (the enemy
 * list holds a counted handle to each), so neither count can reach zero during the loop.
 *
 * 1,048,576 players split among 8 tasks, 16 passes. Each pass every player takes a new target:
 *   default: a hashed pick, enemy ((i * 2654435761 + pass * 0x9E3779B9) * E) >> 32, so the body is nothing but
 *            the re-target (the worst case for the counts);
 *   -DSEARCH=1: the nearest of the E enemies to the player, as probes/y22 writes it (E = 64 only).
 * -DE=64 (hot: 8 threads change 64 counts) or -DE=65536 (cold).
 *
 *   -DMODEL=0  plain C: a pointer store, no counts (not safe; the floor)
 *   -DMODEL=1  the saved rule: in the task, an atomic increment of the new target and an atomic decrement of
 *              the old one (with the test for zero, which never fires here): exactly what Rust's
 *              `p.target = Some(best.clone())` on an `Arc` does (rust/r10_parallel_retarget.rs compiles)
 *   -DMODEL=2  deferred: the task records (old, new) in its own buffer with plain stores; after the join the
 *              parent applies the count changes with plain instructions
 *   -DMODEL=3  netted: each task adds +1/-1 into a small table of its own (1,024 entries, keyed by the object's
 *              address); an entry pushed out of the table, and every entry when the task's loop ends, is applied
 *              to the object's count: an increase at once with an atomic add, a decrease only after every task
 *              has finished its loop (a barrier), with an atomic subtract that may free. No count can reach
 *              zero while any task is still in its loop, because no decrease is applied before the barrier.
 * Models 1 to 3 skip the count changes when the new target is the old one (no count changes then).
 * Checksum: the sum of every player's target index, plus every enemy's final count (equal in models 1 and 2).
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n -DE=64 [-DSEARCH=1] retarget.c -o bin/retarget_n_e64[s].exe */
#include "../../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#ifndef E
#define E 64
#endif
#ifndef SEARCH
#define SEARCH 0
#endif
#include <math.h>
#define N (1u << 20)
#define TASKS 8
#define PASSES 16

typedef struct { uint32_t count; uint32_t type_index; double x; int64_t hp, more; } Enemy;       /* 32 bytes */
typedef struct Player { uint32_t count; uint32_t type_index; double x; Enemy *target; int64_t more; } Player;
static Enemy *enemies;
static Player *players;
static int pass_no;

static __declspec(noinline) void free_enemy(Enemy *e) { e->hp = -1; }   /* never reached: the list holds each */

typedef struct { size_t lo, hi; Enemy **log; size_t n; } Part;
#if MODEL == 3
#define TBL 1024
typedef struct { Enemy *key; int64_t delta; } Ent;
typedef struct { Enemy *e; int64_t n; } Neg;
static __declspec(thread) Ent tbl[TBL];
static __declspec(thread) Neg *negs;
static __declspec(thread) size_t nneg;
static volatile LONG at_barrier;
static __declspec(noinline) void evict(Ent *t) {
    if (t->delta > 0) __atomic_fetch_add(&t->key->count, (uint32_t)t->delta, __ATOMIC_RELAXED);
    else if (t->delta < 0) { negs[nneg].e = t->key; negs[nneg].n = -t->delta; nneg++; }
    t->key = 0; t->delta = 0;
}
static inline void note(Enemy *e, int64_t d) {
    Ent *t = &tbl[((uintptr_t)e >> 5) & (TBL - 1)];
    if (t->key != e) { if (t->key) evict(t); t->key = e; }
    t->delta += d;
}
#endif
static DWORD WINAPI work(LPVOID arg) {
    Part *pt = (Part *)arg;
    Player *ps = players; Enemy *es = enemies;
    uint32_t salt = (uint32_t)pass_no * 0x9E3779B9u;
    size_t n = 0;
    for (size_t i = pt->lo; i < pt->hi; i++) {
        Player *p = &ps[i];
#if SEARCH
        double px = p->x + (double)pass_no;
        Enemy *nw = &es[0];
        for (int k = 1; k < E; k++) if (fabs(es[k].x - px) < fabs(nw->x - px)) nw = &es[k];
#else
        Enemy *nw = &es[(uint32_t)(((uint64_t)((uint32_t)i * 2654435761u + salt) * E) >> 32)];
#endif
        Enemy *old = p->target;
#if MODEL == 1
        p->target = nw;
        if (nw != old) {                                      /* the same handle stored again changes no count */
            __atomic_fetch_add(&nw->count, 1u, __ATOMIC_RELAXED);
            if (__atomic_fetch_sub(&old->count, 1u, __ATOMIC_ACQ_REL) == 1u) free_enemy(old);
        }
#elif MODEL == 2
        p->target = nw;
        if (nw != old) { pt->log[n++] = old; pt->log[n++] = nw; }
#elif MODEL == 3
        p->target = nw;
        if (nw != old) { note(nw, 1); note(old, -1); }
#else
        p->target = nw;
#endif
    }
    pt->n = n;
#if MODEL == 3
    for (int k = 0; k < TBL; k++) if (tbl[k].key) evict(&tbl[k]);
    InterlockedIncrement(&at_barrier);                       /* every task has finished its loop */
    while (at_barrier < TASKS) YieldProcessor();
    for (size_t j = 0; j < nneg; j++)
        if (__atomic_fetch_sub(&negs[j].e->count, (uint32_t)negs[j].n, __ATOMIC_ACQ_REL) == (uint32_t)negs[j].n) free_enemy(negs[j].e);
#endif
    return 0;
}
#if MODEL == 3
static DWORD WINAPI work3(LPVOID arg) {
    negs = (Neg *)malloc(2 * (N / TASKS) * sizeof(Neg)); nneg = 0;
    if (!negs) abort();
    DWORD r = work(arg);
    free(negs);
    return r;
}
#define WORK work3
#else
#define WORK work
#endif

int main(void) {
    h_start();
    enemies = (Enemy *)calloc(E, sizeof(Enemy));
    players = (Player *)calloc(N, sizeof(Player));
    if (!enemies || !players) abort();
    for (uint32_t k = 0; k < E; k++) { enemies[k].count = 1; enemies[k].x = (double)k * (double)(N / E); }     /* the enemy list's handle */
    for (uint32_t i = 0; i < N; i++) { players[i].count = 1; players[i].x = i; players[i].target = &enemies[i % E]; enemies[i % E].count++; }
    Part parts[TASKS];
    for (int k = 0; k < TASKS; k++) {
        parts[k].lo = (size_t)k * (N / TASKS); parts[k].hi = parts[k].lo + N / TASKS;
        parts[k].log = (Enemy **)malloc(2 * (N / TASKS) * sizeof(Enemy *));
        if (!parts[k].log) abort();
    }
    h_start();
    for (pass_no = 0; pass_no < PASSES; pass_no++) {
        HANDLE th[TASKS];
#if MODEL == 3
        at_barrier = 0;
#endif
        for (int k = 0; k < TASKS; k++) { th[k] = CreateThread(NULL, 0, WORK, &parts[k], 0, NULL); if (!th[k]) abort(); }
        WaitForMultipleObjects(TASKS, th, TRUE, INFINITE);
        for (int k = 0; k < TASKS; k++) CloseHandle(th[k]);
#if MODEL == 2
        for (int k = 0; k < TASKS; k++)                       /* after the join: plain count changes */
            for (size_t j = 0; j < parts[k].n; j += 2) {
                parts[k].log[j + 1]->count++;
                if (--parts[k].log[j]->count == 0) free_enemy(parts[k].log[j]);
            }
#endif
    }
    uint64_t sum = 0;
    for (uint32_t i = 0; i < N; i++) sum += (uint64_t)(players[i].target - enemies);
#if MODEL != 0
    for (uint32_t k = 0; k < E; k++) sum += enemies[k].count;
#else
    for (uint32_t k = 0; k < E; k++) sum += 1;                 /* what the counts would be: N handles + E */
    sum += N;
#endif
    static const char *names[] = {"retarget-0-plainC", "retarget-1-atomic-in-task", "retarget-2-deferred-to-join",
                                  "retarget-3-netted"};
    char name[64]; snprintf(name, sizeof name, "%s-e%d%s", names[MODEL], E, SEARCH ? "-search" : "");
    h_report(name, sum);
    return 0;
}
