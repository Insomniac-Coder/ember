/* claim2.c: saving "claimed objects" (A15, section 4.7 of phase3b-threads.md).
 *
 * Same workload as ../../bench/claim.c: 8 tasks each change the class objects of their part of a list of
 * 1,048,576 shuffled handles, 64 passes (`p.hp -= 1; p.x += p.hp & 1`). Python: ex.map(hurt, players).
 *
 * The idea saved: a count of 1 already proves that the list's handle is the only counted handle, so no
 * other task can reach the object. Only objects with a count above 1 need a claim.
 *
 *   -DMODEL=0  plain C, no check (claim.c model 0)
 *   -DMODEL=1  an atomic claim on every object (claim.c model 1, the discarded answer)
 *   -DMODEL=3  the count test: a relaxed load of the count (a plain `mov` on x86, on the cache line the task
 *              writes anyway); 1 means plain writes; above 1 means take the object's claim byte with an
 *              atomic exchange, waiting while another task holds it, and release it after
 *   -DMODEL=4  as 3, but one object in 16 has a second handle elsewhere (count 2), so it takes the claim
 *   -DMODEL=5  correctness: as 3, and one slot in 16 holds a handle to an object another slot also holds
 *              (count 2): two tasks may reach one object; the program checks that no decrement is lost
 *   -DMODEL=6  as 5 with model 0's code (no check): shows what the check prevents (a data race in C;
 *              compiled as plain read-modify-write on x86, it can lose decrements)
 *   -DMODEL=7  model 3 written better: the list's buffer and the part's end are kept in locals (the list
 *              cannot be reshaped during the scope, so the compiler knows its buffer stays put) and the
 *              claim path is a separate function, so the loop holds only the load, the compare and the body
 *   -DMODEL=8  model 7 with one object in 16 counted twice (model 4's mix)
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n claim2.c -o bin/claim2_n.exe */
#include "../../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#define N (1u << 20)
#define TASKS 8
#define PASSES 64
#ifndef SWEEPS
#define SWEEPS 1   /* -DSWEEPS=8: each task walks its part 8 times per pass, so thread starts weigh less */
#endif

typedef struct { uint32_t strong; uint8_t claim, pad[3]; int32_t hp, x; int64_t more[2]; } Player;  /* 32 bytes */
static Player **players;

static __declspec(noinline) void fail(const char *what) { fprintf(stderr, "panic: %s\n", what); exit(101); }

static inline void body(Player *p) { p->hp -= 1; p->x += p->hp & 1; }

typedef struct { size_t lo, hi; } Part;
#if MODEL == 7 || MODEL == 8
static __declspec(noinline) void claimed_body(Player *p) {
    while (__atomic_exchange_n(&p->claim, (uint8_t)1, __ATOMIC_ACQUIRE)) YieldProcessor();
    body(p);
    __atomic_store_n(&p->claim, (uint8_t)0, __ATOMIC_RELEASE);
}
static DWORD WINAPI work(LPVOID arg) {
    Part *pt = (Part *)arg;
    Player **list = players;
    size_t lo = pt->lo, hi = pt->hi;
    for (int sw = 0; sw < SWEEPS; sw++)
    for (size_t i = lo; i < hi; i++) {
        Player *p = list[i];
        if (__atomic_load_n(&p->strong, __ATOMIC_RELAXED) == 1) body(p);
        else claimed_body(p);
    }
    return 0;
}
#else
static DWORD WINAPI work(LPVOID arg) {
    Part *pt = (Part *)arg;
    for (int sw = 0; sw < SWEEPS; sw++)
    for (size_t i = pt->lo; i < pt->hi; i++) {
        Player *p = players[i];
#if MODEL == 1
        if (__atomic_exchange_n(&p->claim, (uint8_t)1, __ATOMIC_ACQUIRE)) fail("one object in two tasks");
        body(p);
        __atomic_store_n(&p->claim, (uint8_t)0, __ATOMIC_RELEASE);
#elif MODEL == 3 || MODEL == 4 || MODEL == 5
        if (__atomic_load_n(&p->strong, __ATOMIC_RELAXED) == 1) {
            body(p);                                         /* the only handle: no other task can reach it */
        } else {
            while (__atomic_exchange_n(&p->claim, (uint8_t)1, __ATOMIC_ACQUIRE)) YieldProcessor();
            body(p);
            __atomic_store_n(&p->claim, (uint8_t)0, __ATOMIC_RELEASE);
        }
#else
        body(p);
#endif
    }
    return 0;
}
#endif

int main(void) {
    h_start();
    h_seed(11);
    Player *store = (Player *)calloc(N, sizeof(Player));
    players = (Player **)malloc(N * sizeof(Player *));
    if (!store || !players) abort();
    for (uint32_t i = 0; i < N; i++) { store[i].strong = 1; store[i].hp = 1000000; store[i].x = (int32_t)i; players[i] = &store[i]; }
    for (uint32_t i = N - 1; i > 0; i--) { uint32_t j = h_rand() % (i + 1); Player *t = players[i]; players[i] = players[j]; players[j] = t; }
#if MODEL == 4 || MODEL == 8
    for (uint32_t i = 0; i < N; i += 16) players[i]->strong = 2;           /* a handle held elsewhere too */
#endif
#if MODEL == 5 || MODEL == 6
    /* One slot in 16 is re-pointed at the object of a slot half the list away (another task's part). */
    for (uint32_t i = 0; i < N; i++) store[i].strong = 0;
    for (uint32_t i = 0; i < N; i += 16) players[i] = players[(i + N / 2 + 8) % N];
    for (uint32_t i = 0; i < N; i++) players[i]->strong += 1;
    int64_t expect_hp = 0;
    for (uint32_t i = 0; i < N; i++) expect_hp += store[i].hp - (int64_t)PASSES * SWEEPS * store[i].strong;
#endif
    h_start();                                         /* time the passes only */
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
    int64_t got_hp = 0;
    for (uint32_t i = 0; i < N; i++) got_hp += store[i].hp;
    fprintf(stderr, "%s: decrements lost = %lld\n", MODEL == 5 ? "claim2-5" : "claim2-6", (long long)(got_hp - expect_hp));
    sum = (uint64_t)(got_hp - expect_hp);              /* the checksum is the number of lost decrements */
#endif
    static const char *names[] = {"claim2-0-plainC", "claim2-1-atomic-claims", "", "claim2-3-count-test",
                                  "claim2-4-count-test-1in16-shared", "claim2-5-dups-checked", "claim2-6-dups-unchecked",
                                  "claim2-7-count-test-tight", "claim2-8-count-test-tight-1in16"};
    h_report(names[MODEL], sum);
    return 0;
}
