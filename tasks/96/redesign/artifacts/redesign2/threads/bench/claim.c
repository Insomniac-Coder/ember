/* claim.c: 8 tasks each change the class objects of their part of a list of handles (A15, "claimed
 * objects").
 *
 * Ember (proposed):  @parallel
 *                    for p in players:           # players: Array[Player], Player a class
 *                        p.hp -= 1; p.x += p.hp & 1
 * Python:            with ThreadPoolExecutor(8) as ex: ex.map(hurt, chunks(players, 8))
 *
 * 1,048,576 handles to 1,048,576 objects (shuffled, so the objects are scattered as handles usually
 * are), 64 passes. The list could hold one object twice, which would make two tasks write one object;
 * the models differ in how that is ruled out. Same checksum in every model.
 *
 *   -DMODEL=0  plain C: no check (the C programmer promises no duplicates)
 *   -DMODEL=1  claims: before a task writes an object it swaps a claim byte in the object's header to 1
 *              (an atomic exchange with acquire order; a 1 already there means another task holds it: panic),
 *              and stores 0 with release order when done with it. The pair is also what makes a later
 *              task's writes to the same object ordered after this one's.
 *   -DMODEL=2  claims taken by the parent before the tasks start (plain stores, one pass in one thread)
 *              and cleared by each task as it finishes each object (plain stores); the join orders them
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n claim.c -o claim_n.exe */
#include "../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#define N (1u << 20)
#define TASKS 8
#define PASSES 64

typedef struct { uint32_t strong; uint8_t claim, pad[3]; int32_t hp, x; int64_t more[2]; } Player;  /* 32 bytes */
static Player **players;

static __declspec(noinline) void fail(const char *what) { fprintf(stderr, "panic: %s\n", what); exit(101); }

typedef struct { size_t lo, hi; } Part;
static DWORD WINAPI work(LPVOID arg) {
    Part *pt = (Part *)arg;
    for (size_t i = pt->lo; i < pt->hi; i++) {
        Player *p = players[i];
#if MODEL == 1
        if (__atomic_exchange_n(&p->claim, (uint8_t)1, __ATOMIC_ACQUIRE)) fail("one object in two tasks");
#endif
        p->hp -= 1;
        p->x += p->hp & 1;
#if MODEL == 1
        __atomic_store_n(&p->claim, (uint8_t)0, __ATOMIC_RELEASE);
#elif MODEL == 2
        p->claim = 0;
#endif
    }
    return 0;
}

int main(void) {
    h_start();
    h_seed(11);
    Player *store = (Player *)calloc(N, sizeof(Player));
    players = (Player **)malloc(N * sizeof(Player *));
    if (!store || !players) abort();
    for (uint32_t i = 0; i < N; i++) { store[i].strong = 1; store[i].hp = 1000000; store[i].x = (int32_t)i; players[i] = &store[i]; }
    for (uint32_t i = N - 1; i > 0; i--) { uint32_t j = h_rand() % (i + 1); Player *t = players[i]; players[i] = players[j]; players[j] = t; }
    h_start();                                         /* time the passes only */
    Part parts[TASKS];
    for (int pass = 0; pass < PASSES; pass++) {
#if MODEL == 2
        for (size_t i = 0; i < N; i++) { if (players[i]->claim) fail("one object in two tasks"); players[i]->claim = 1; }
#endif
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
    static const char *names[] = {"claim-0-plainC", "claim-1-atomic-claims", "claim-2-parent-claims"};
    h_report(names[MODEL], sum);
    return 0;
}
