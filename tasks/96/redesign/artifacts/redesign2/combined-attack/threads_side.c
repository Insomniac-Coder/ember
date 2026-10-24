/* threads_side.c: the threads group's netted counts (phase3b-threads-attack.md 3.1) meet the combined
   model's count word and side records (phase3b-combined.md 3.4, 4.3). Problem 2 of
   tasks/96/redesign/phase3b-combined-attack.md.

   The Ember program (the combined model's syntax):

       class Player:
           hp: int = 100
       class Watcher:
           target: Player                          # a strong handle
       class Camera:
           target: weak Player = None

       players = [Player() for _ in range(8)]
       cams = [Camera() for _ in range(64)]
       for i, c in enumerate(cams): c.target = players[i % 8]      # every player is weakly referenced
       ws = [Watcher(target=players[0]) for _ in range(64)]
       @parallel                                                    # 4 tasks
       for i, w in enumerate(ws):
           w.target = players[1 + i % 7]           # netted: +1 new target at loop end, -1 old after the barrier
           cams[i].target = players[(i + 3) % 8]   # a weak store in a task
       players.clear(); ws.clear(); cams.clear()   # everything is freed exactly once

   As written (default build): a Player that a weak field points at keeps its counts in a side record
   and its count word holds SIDE and the record's index (combined/w3.c). The netted increase is the
   threads attack's `__atomic_fetch_add(&count, delta)` (threads/attack2/bench/retarget.c line 59) and
   the decrease its `__atomic_fetch_sub(&count, n) == n` (line 106): on this count word they add to and
   subtract from the side-record INDEX. The weak store in a task runs combined/w3.c's weak_retain and
   weak_release: plain increments of a record other tasks also change, and the global table's realloc.
   -DSAVED: the fix of problem 2 (with problem 5's layout): the strong count never leaves the header
   (only a count past 28 bits would); the weak count lives beside the object (here an array indexed by
   the player's slot, standing for its slab chunk's array); weak count changes in a task are netted
   exactly like strong ones; a count reaching zero after the barrier tests the weak count beside it.
   Objects are malloc'd one by one and freed with free() so AddressSanitizer sees every misuse.
     clang -O1 -g -fsanitize=address threads_side.c -o threads_side_as_written.exe
     clang -O1 -g -fsanitize=address -DSAVED threads_side.c -o threads_side_saved.exe
   -DRACE (as written only): eight threads each make the first weak reference to 100,000 objects of
   their own at the same time (allowed by the scope rule: each task writes only its own cameras), then
   checks every side record. */
#include "../../redesign/bench/harness.h"
#include <string.h>

#define SIDE (1u << 29)
#define DEAD (1u << 28)
#define LOW (DEAD - 1u)
#define TASKS 4
#define NP 8
#define NW 64

typedef struct Player { uint32_t rc; int32_t id; int64_t hp; } Player;
typedef struct Watcher { Player *target; } Watcher;
typedef struct Camera { Player *target; } Camera;          /* weak */
static Player *players[NP];
static Watcher ws[NW];
static Camera cams[NW];
static int64_t freed_count;

static void free_player(Player *p) { p->hp = -1; free(p); __atomic_fetch_add(&freed_count, 1, __ATOMIC_RELAXED); }

#ifndef SAVED
/* ---- combined/w3.c's side records, copied unchanged ---- */
typedef struct { uint64_t strong, weak; } SideRec;
static SideRec *side; static uint32_t side_len, side_cap, side_free_head = UINT32_MAX;
static uint32_t side_alloc(void) {
    if (side_free_head != UINT32_MAX) { uint32_t i = side_free_head; side_free_head = (uint32_t)side[i].weak; return i; }
    if (side_len == side_cap) {
        side_cap = side_cap ? side_cap * 2 : 1024;
        side = realloc(side, side_cap * sizeof(SideRec)); if (!side) abort();
    }
    return side_len++;
}
static void side_release(uint32_t i) { side[i].weak = side_free_head; side_free_head = i; }
static void weak_retain(Player *t) {
    if (!(t->rc & SIDE)) {
        uint32_t i = side_alloc();
        side[i].strong = t->rc & LOW; side[i].weak = 0;
        t->rc = (t->rc & ~LOW) | SIDE | i;
    }
    side[t->rc & LOW].weak++;
}
static void weak_release(Player *t) {
    uint32_t i = t->rc & LOW;
    if (--side[i].weak != 0) return;
    if (t->rc & DEAD) { side_release(i); free_player(t); }
    else if (side[i].strong <= LOW) { t->rc = (t->rc & ~(LOW | SIDE)) | (uint32_t)side[i].strong; side_release(i); }
}
static void release(Player *o) {                         /* parent thread, after the scope */
    if (!(o->rc & SIDE)) { if ((--o->rc & LOW) == 0) free_player(o); return; }
    uint32_t i = o->rc & LOW;
    if (i >= side_len) { printf("as written: %d's count word names side record %u, past the %u that exist\n", o->id, i, side_len); exit(3); }
    if (--side[i].strong != 0) return;
    o->rc |= DEAD; side[i].weak++; weak_release(o);
}
static void weak_store(Camera *c, Player *t) { if (t) weak_retain(t); Player *old = c->target; c->target = t; if (old) weak_release(old); }
static void netted_add(Player *p, uint32_t d) { __atomic_fetch_add(&p->rc, d, __ATOMIC_RELAXED); }          /* retarget.c line 59 */
static void netted_sub(Player *p, uint32_t n) { if (__atomic_fetch_sub(&p->rc, n, __ATOMIC_ACQ_REL) == n) free_player(p); }  /* line 106 */
#define WEAK_IN_TASK(c, t) weak_store(c, t)
#else
/* ---- the fix: strong count in the header, weak count beside the object ---- */
/* The weak count beside the object counts the weak fields plus one for all the strong handles together
   while any weak field exists (Rust's Arc keeps the same extra one), so exactly one decrement reaches
   zero and frees the block, whichever side lets go last. */
static uint32_t weak_beside[NP];                          /* stands for the slab chunk's array of weak counts */
static void weak_inc(Player *t, uint32_t d) {             /* 0 -> d + 1: the first weak field adds the extra one */
    uint32_t old = __atomic_load_n(&weak_beside[t->id], __ATOMIC_RELAXED);
    while (!__atomic_compare_exchange_n(&weak_beside[t->id], &old, old ? old + d : d + 1, 1, __ATOMIC_ACQ_REL, __ATOMIC_RELAXED)) {}
}
static void weak_dec(Player *t, uint32_t n) {
    if (__atomic_fetch_sub(&weak_beside[t->id], n, __ATOMIC_ACQ_REL) == n) free_player(t);
}
static void finish_zero(Player *p) {                      /* the strong count reached zero: no strong handle is left, */
    if (__atomic_load_n(&weak_beside[p->id], __ATOMIC_ACQUIRE) == 0) { free_player(p); return; }   /* so no new weak one */
    __atomic_fetch_or(&p->rc, DEAD, __ATOMIC_RELEASE);    /* weak reads now give None; the block waits */
    weak_dec(p, 1);                                       /* the strong side's extra one */
}
static void release(Player *o) { if (((o->rc - 1) & LOW) == 0) { o->rc = 0; finish_zero(o); } else o->rc--; }
static void weak_store(Camera *c, Player *t) { if (t) weak_inc(t, 1); Player *old = c->target; c->target = t; if (old) weak_dec(old, 1); }
static void netted_add(Player *p, uint32_t d) {
    uint32_t old = __atomic_fetch_add(&p->rc, d, __ATOMIC_RELAXED);
    if (((old & LOW) + d) > LOW - 1) { fprintf(stderr, "count past 28 bits\n"); exit(101); }   /* the attack's overflow check */
}
static void netted_sub(Player *p, uint32_t n) {
    uint32_t old = __atomic_fetch_sub(&p->rc, n, __ATOMIC_ACQ_REL);
    if ((old & LOW) == n) finish_zero(p);                 /* compare the count bits, not the flags */
}
#define WEAK_IN_TASK(c, t) netted_weak(c, t)
#endif

/* ---- one task: the netted table (a few entries here), as retarget.c MODEL 3 ---- */
typedef struct { Player *key; int64_t strong, weak; } Ent;
typedef struct { int lo, hi; Ent tbl[NP]; } Part;
static volatile LONG at_barrier;
#ifdef SAVED
static __declspec(thread) Part *cur_part;
static void netted_weak(Camera *c, Player *t) {          /* the store is plain (c is the task's own camera); counts netted */
    Player *old = c->target; c->target = t;
    cur_part->tbl[t->id].key = t; cur_part->tbl[t->id].weak++;
    if (old) { cur_part->tbl[old->id].key = old; cur_part->tbl[old->id].weak--; }
}
#endif
static DWORD WINAPI task(LPVOID arg) {
    Part *pt = arg;
#ifdef SAVED
    cur_part = pt;
#endif
    for (int i = pt->lo; i < pt->hi; i++) {
        Player *nw = players[1 + i % 7], *old = ws[i].target;
        ws[i].target = nw;
        pt->tbl[nw->id].key = nw; pt->tbl[nw->id].strong++;
        pt->tbl[old->id].key = old; pt->tbl[old->id].strong--;
        WEAK_IN_TASK(&cams[i], players[(i + 3) % NP]);
    }
    for (int k = 0; k < NP; k++) {                       /* increases at the loop's end */
        if (pt->tbl[k].strong > 0) netted_add(pt->tbl[k].key, (uint32_t)pt->tbl[k].strong);
#ifdef SAVED
        if (pt->tbl[k].weak > 0) weak_inc(pt->tbl[k].key, (uint32_t)pt->tbl[k].weak);
#endif
    }
    InterlockedIncrement(&at_barrier);
    while (at_barrier < TASKS) YieldProcessor();
    for (int k = 0; k < NP; k++) {                       /* decreases after the barrier */
        if (pt->tbl[k].strong < 0) netted_sub(pt->tbl[k].key, (uint32_t)-pt->tbl[k].strong);
#ifdef SAVED
        if (pt->tbl[k].weak < 0) weak_dec(pt->tbl[k].key, (uint32_t)-pt->tbl[k].weak);
#endif
    }
    return 0;
}

#ifdef RACE
static Player *race_objs[8][100000];
static Camera race_cams[8][100000];
static DWORD WINAPI race_task(LPVOID arg) {
    int t = (int)(intptr_t)arg;
    for (int i = 0; i < 100000; i++) weak_store(&race_cams[t][i], race_objs[t][i]);
    return 0;
}
#endif

int main(void) {
    h_start();
#ifdef RACE
    for (int t = 0; t < 8; t++) for (int i = 0; i < 100000; i++) { Player *p = calloc(1, sizeof *p); p->rc = 1; p->id = i; race_objs[t][i] = p; }
    HANDLE rth[8];
    for (int t = 0; t < 8; t++) rth[t] = CreateThread(NULL, 0, race_task, (LPVOID)(intptr_t)t, 0, NULL);
    WaitForMultipleObjects(8, rth, TRUE, INFINITE);
    int64_t bad = 0, shared_idx = 0;
    static uint8_t seen[1u << 20];
    for (int t = 0; t < 8; t++) for (int i = 0; i < 100000; i++) {
        Player *p = race_objs[t][i];
        uint32_t idx = p->rc & LOW;
        if (!(p->rc & SIDE) || idx >= side_len || side[idx].weak != 1 || side[idx].strong != 1) bad++;
        if (idx < (1u << 20)) { if (seen[idx]) shared_idx++; seen[idx] = 1; }
    }
    printf("race: 800000 first weak stores from 8 tasks; side records allocated %u; objects with a wrong record %lld; records given to two objects %lld\n",
           side_len, (long long)bad, (long long)shared_idx);
    return 0;
#else
    for (int i = 0; i < NP; i++) { players[i] = calloc(1, sizeof(Player)); players[i]->rc = 1; players[i]->id = i; players[i]->hp = 100; }
    for (int i = 0; i < NW; i++) weak_store(&cams[i], players[i % NP]);                    /* every player weakly referenced */
    for (int i = 0; i < NW; i++) {                                                           /* Watcher(target=players[0]) */
        ws[i].target = players[0];
#ifndef SAVED
        if (players[0]->rc & SIDE) side[players[0]->rc & LOW].strong++; else players[0]->rc++;
#else
        players[0]->rc++;
#endif
    }
    Part parts[TASKS]; memset(parts, 0, sizeof parts);
    HANDLE th[TASKS];
    for (int k = 0; k < TASKS; k++) { parts[k].lo = k * (NW / TASKS); parts[k].hi = parts[k].lo + NW / TASKS; th[k] = CreateThread(NULL, 0, task, &parts[k], 0, NULL); }
    WaitForMultipleObjects(TASKS, th, TRUE, INFINITE);
    /* after the scope: what each player's count should be: 1 (players) + the watchers targeting it */
    int bad = 0;
    for (int p = 0; p < NP; p++) {
        uint32_t expect = 1; for (int i = 0; i < NW; i++) expect += ws[i].target == players[p];
#ifndef SAVED
        uint32_t rc = players[p]->rc, got = (rc & SIDE) ? ((rc & LOW) < side_len ? (uint32_t)side[rc & LOW].strong : 0xFFFFFFFFu) : (rc & LOW);
        if (got != expect) { bad++; printf("as written: player %d count word %08x (SIDE %u, index %u); its count should be %u, it reads %d\n",
                                              p, rc, (rc >> 29) & 1, rc & LOW, expect, (int)got); }
#else
        uint32_t got = players[p]->rc & LOW;
        if (got != expect) { bad++; printf("saved: player %d count %u, should be %u\n", p, got, expect); }
#endif
    }
    /* players.clear(); ws.clear(); cams.clear(): every player must be freed exactly once, after its last use */
    for (int p = 0; p < NP; p++) release(players[p]);
    for (int i = 0; i < NW; i++) release(ws[i].target);
    for (int i = 0; i < NW; i++) weak_store(&cams[i], NULL);
    printf("%s: %d players with a wrong count after the scope; players freed %lld of %d\n",
#ifdef SAVED
           "saved",
#else
           "as written",
#endif
           bad, (long long)freed_count, NP);
    return 0;
#endif
}
