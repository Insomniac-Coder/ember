/* claim3.c: the first attack's saved "each task changes its own class object" (phase3b-threads-attack-first-run.md
 * 3.5), measured again with the lean class's real header (phase3b-ownership.md 2.3: a 32-bit count whose top
 * bits hold the flags, and a 32-bit type index). The first attack's claim2.c put a separate claim byte where the
 * type index goes; the lean header has no free byte, so the claim must be a bit of the count word (the first
 * attack's own suggestion, "bit 30"). Then taking and dropping it are atomic read-modify-writes of the count word.
 *
 * Same workload as claim2.c / ../../bench/claim.c: 8 tasks each change the objects of their part of a list of
 * 1,048,576 shuffled handles, 64 passes (`p.hp -= 1; p.x += p.hp & 1`). Python: ex.map(hurt, players).
 *
 *   -DMODEL=0  plain C, no check, same 32-byte object
 *   -DMODEL=1  the count test; every count is 1 (the list is the objects' only holder)
 *   -DMODEL=2  the count test; one object in 16 also held elsewhere (count 2): it takes the claim
 *   -DMODEL=3  the count test; every object also held elsewhere (count 2): the list is a second list of the
 *              same objects (a filtered list, a selection, a spatial grid), so every element takes the claim
 *   -DMODEL=4  an atomic claim on every element with no count test (the group's discarded answer, this header)
 *   -DMODEL=5  correctness: one slot in 16 holds an object another task's part also holds, and the tasks also
 *              retain other objects with atomic increments while the claims run (the first attack's rule 4b:
 *              "a retain of a parent object is emitted as an atomic increment"); the claim is dropped with an
 *              atomic AND. Prints decrements lost and count increments lost.
 *   -DMODEL=6  as 5, but the claim is dropped with a plain read-modify-write of the count word (what a
 *              separate claim byte allowed: a plain store)
 * Claim path: take = lock bts (atomic OR of bit 30), wait while another task holds it; drop = lock and.
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n [-DSWEEPS=8] claim3.c -o bin/claim3[s]_n.exe */
#include "../../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#define N (1u << 20)
#define TASKS 8
#define PASSES 64
#define CLAIM (1u << 30)
#ifndef SWEEPS
#define SWEEPS 1   /* -DSWEEPS=8: each task walks its part 8 times per pass, so thread starts weigh less (claim2.c's option) */
#endif

typedef struct { uint32_t count; uint32_t type_index; int32_t hp, x; int64_t more[2]; } Player;  /* 32 bytes */
static Player **players;
static volatile LONG64 retains_done;

static inline void body(Player *p) { p->hp -= 1; p->x += p->hp & 1; }

static __declspec(noinline) void claimed_body(Player *p) {
    while (__atomic_fetch_or(&p->count, CLAIM, __ATOMIC_ACQUIRE) & CLAIM) YieldProcessor();
    body(p);
#if MODEL == 6
    { volatile uint32_t *c = &p->count; *c = *c & ~CLAIM; }     /* plain read-modify-write: not atomic */
#else
    __atomic_fetch_and(&p->count, ~CLAIM, __ATOMIC_RELEASE);
#endif
}

typedef struct { size_t lo, hi; } Part;
static DWORD WINAPI work(LPVOID arg) {
    Part *pt = (Part *)arg;
    Player **list = players;
    size_t lo = pt->lo, hi = pt->hi;
    int64_t my_retains = 0;
    for (int sw = 0; sw < SWEEPS; sw++)
    for (size_t i = lo; i < hi; i++) {
        Player *p = list[i];
#if MODEL == 0
        body(p);
#elif MODEL == 4
        claimed_body(p);
#else
        if (__atomic_load_n(&p->count, __ATOMIC_RELAXED) == 1) body(p);   /* the only handle, no flag, no claim */
        else claimed_body(p);
#endif
#if MODEL == 5 || MODEL == 6
        if ((i & 15) == 4) {                         /* retain an object that is claimed elsewhere (slot i + 4) */
            __atomic_fetch_add(&list[(i + 4) % N]->count, 1u, __ATOMIC_RELAXED);
            my_retains++;
        }
#endif
    }
    InterlockedAdd64(&retains_done, my_retains);
    return 0;
}

int main(void) {
    h_start();
    h_seed(11);
    Player *store = (Player *)calloc(N, sizeof(Player));
    players = (Player **)malloc(N * sizeof(Player *));
    if (!store || !players) abort();
    for (uint32_t i = 0; i < N; i++) { store[i].count = 1; store[i].type_index = 7; store[i].hp = 1000000; store[i].x = (int32_t)i; players[i] = &store[i]; }
    for (uint32_t i = N - 1; i > 0; i--) { uint32_t j = h_rand() % (i + 1); Player *t = players[i]; players[i] = players[j]; players[j] = t; }
#if MODEL == 2
    for (uint32_t i = 0; i < N; i += 16) players[i]->count = 2;
#elif MODEL == 3
    for (uint32_t i = 0; i < N; i++) players[i]->count = 2;
#endif
#if MODEL == 5 || MODEL == 6
    for (uint32_t i = 0; i < N; i++) store[i].count = 0;
    for (uint32_t i = 0; i < N; i += 16) players[i] = players[(i + N / 2 + 8) % N];
    for (uint32_t i = 0; i < N; i++) players[i]->count += 1;
    int64_t expect_hp = 0, count0 = 0;
    for (uint32_t i = 0; i < N; i++) { expect_hp += store[i].hp - (int64_t)PASSES * SWEEPS * store[i].count; count0 += store[i].count; }
#endif
    h_start();
    Part parts[TASKS];
    for (int pass = 0; pass < PASSES; pass++) {
        HANDLE th[TASKS];
        for (int k = 0; k < TASKS; k++) {
            parts[k].lo = (size_t)k * (N / TASKS); parts[k].hi = parts[k].lo + N / TASKS;
            th[k] = CreateThread(NULL, 0, work, &parts[k], 0, NULL);
            if (!th[k]) abort();
        }
        WaitForMultipleObjects(TASKS, th, TRUE, INFINITE);
        for (int k = 0; k < TASKS; k++) CloseHandle(th[k]);
    }
    uint64_t sum = 0;
    for (uint32_t i = 0; i < N; i++) sum += (uint64_t)(uint32_t)(store[i].hp ^ store[i].x);
#if MODEL == 5 || MODEL == 6
    int64_t got_hp = 0, count1 = 0, claims_left = 0;
    for (uint32_t i = 0; i < N; i++) { got_hp += store[i].hp; count1 += store[i].count & ~CLAIM; claims_left += (store[i].count & CLAIM) != 0; }
    int64_t lost_dec = got_hp - expect_hp, lost_inc = (count0 + retains_done) - count1;
    fprintf(stderr, "claim3-%d: decrements lost = %lld, count increments lost = %lld (of %lld), claims left set = %lld\n",
            MODEL, (long long)lost_dec, (long long)lost_inc, (long long)retains_done, (long long)claims_left);
    sum = (uint64_t)(lost_dec * 1000000 + lost_inc);
#endif
    static const char *names[] = {"claim3-0-plainC", "claim3-1-count-test-sole-owner", "claim3-2-count-test-1in16-shared",
                                  "claim3-3-count-test-all-shared", "claim3-4-atomic-claim-every-element",
                                  "claim3-5-dups-retains-atomic-drop", "claim3-6-dups-retains-plain-drop"};
    h_report(names[MODEL], sum);
    return 0;
}
