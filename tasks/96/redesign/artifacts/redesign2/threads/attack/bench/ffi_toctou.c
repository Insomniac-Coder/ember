/* ffi_toctou.c: a callback unregistered on one thread while C is running it on another (A20, threads(any)).
 *
 * The C library's own thread W calls the callback once. The callback reads its captured `Game*`, signals that
 * it is running, works for 200 ms, then adds 1 to `game.score`. The main thread waits for the signal and
 * unregisters the callback; unregistering frees the closure's state, which releases the last handle to the
 * Game. Here "free" overwrites the Game with a poison value (a reused allocation would be overwritten), so a
 * use after free shows up as the callback writing into poison instead of crashing.
 *
 *   -DFIX=0  the checked trampoline as proposed (range, generation; threads(any): no thread compare):
 *            the generation check passed before the unregister, so the callback runs on freed state
 *   -DFIX=1  the fix: the trampoline announces the slot in its thread's record (ffi2.c model 7) and
 *            unregistering bumps the generation, calls FlushProcessWriteBuffers, and waits until no record
 *            announces the slot before freeing
 *   -DFIX=2  as 1, and then times 10,000 unregisters while 7 other threads keep calling callbacks
 *            (prints microseconds per unregister); run pinned to the P-cores
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DFIX=n ffi_toctou.c -o bin/ffi_toctou_n.exe */
#include "../../../../redesign/bench/harness.h"

#ifndef FIX
#define FIX 0
#endif
#define POISON (-1000000)

typedef void (*cb_t)(void *);
static __declspec(noinline) void fire(cb_t cb, void *ud, int64_t n) { for (int64_t i = 0; i < n; i++) cb(ud); }
static __declspec(noinline) void fail(const char *what) { fprintf(stderr, "panic: %s\n", what); exit(101); }

typedef struct { int64_t score; } Game;
typedef struct { Game *game; int slow; } Env;
typedef struct { uint32_t generation; uint32_t type; Env env; } Slot;
typedef struct { uint32_t depth, pending; uint64_t h[14]; } ThreadRec;

#define MAX_THREADS 16
static ThreadRec recs[MAX_THREADS];                /* the runtime's records of attached threads */
static volatile LONG n_recs;
static __declspec(thread) ThreadRec *t_rec;
static void attach(void) { t_rec = &recs[InterlockedIncrement(&n_recs) - 1]; }

static Slot table[64];
static HANDLE running_event;
static volatile LONG64 observed_poison;

static void lambda_body(Env *env) {
    Game *g = env->game;
    if (env->slow) {
        SetEvent(running_event);
        Sleep(200);                                  /* the callback's work */
        if (g->score == POISON) InterlockedIncrement64(&observed_poison);
    }
    g->score += 1;
}

static void tramp(void *ud) {
    uintptr_t h = (uintptr_t)ud;
    if ((uint32_t)h >= 64) fail("a callback handle C made up");
    Slot *s = &table[(uint32_t)h];
#if FIX == 0
    if (__atomic_load_n(&s->generation, __ATOMIC_ACQUIRE) != (uint32_t)(h >> 32)) return;
    lambda_body(&s->env);
#else
    ThreadRec *r = t_rec;
    uint32_t d = r->depth;
    if (d >= 14) fail("nested too deep");
    __atomic_store_n(&r->h[d], (uint64_t)h, __ATOMIC_RELAXED);
    __atomic_store_n(&r->depth, d + 1, __ATOMIC_RELAXED);
    __atomic_signal_fence(__ATOMIC_SEQ_CST);
    if (__atomic_load_n(&s->generation, __ATOMIC_RELAXED) == (uint32_t)(h >> 32)) lambda_body(&s->env);
    __atomic_store_n(&r->depth, d, __ATOMIC_RELEASE);
#endif
}

static void unregister(uint32_t slot) {
    __atomic_fetch_add(&table[slot].generation, 1, __ATOMIC_SEQ_CST);
#if FIX >= 1
    FlushProcessWriteBuffers();                      /* the heavy half: every running thread's stores are visible */
    for (LONG i = 0; i < n_recs; i++) {
        ThreadRec *r = &recs[i];
        if (r == t_rec) continue;                    /* this thread's own calls: deferred free (not shown) */
        for (;;) {
            uint32_t d = __atomic_load_n(&r->depth, __ATOMIC_ACQUIRE), busy = 0;
            for (uint32_t k = 0; k < d && k < 14; k++)
                if ((uint32_t)__atomic_load_n(&r->h[k], __ATOMIC_RELAXED) == slot) busy = 1;
            if (!busy) break;
            YieldProcessor();
        }
    }
#endif
    /* free the closure's state: its handle was the Game's last one */
    if (table[slot].env.game) table[slot].env.game->score = POISON;
}

static volatile LONG stop;
static DWORD WINAPI library_thread(LPVOID ud) { attach(); fire(tramp, ud, 1); return 0; }
static DWORD WINAPI busy_thread(LPVOID ud) {
    attach();
    while (!stop) fire(tramp, ud, 1000);
    return 0;
}

int main(void) {
    h_start();
    attach();
    running_event = CreateEventA(NULL, TRUE, FALSE, NULL);
    Game *g = (Game *)calloc(1, sizeof(Game));
    table[5] = (Slot){ 9, 17, { g, 1 } };
    HANDLE w = CreateThread(NULL, 0, library_thread, (LPVOID)(((uintptr_t)9 << 32) | 5), 0, NULL);
    WaitForSingleObject(running_event, INFINITE);    /* C's thread is inside the callback */
    unregister(5);                                   /* the main thread drops the token */
    WaitForSingleObject(w, INFINITE);
    fprintf(stderr, "FIX=%d: the callback wrote into freed state: %s\n", FIX, observed_poison ? "YES" : "no");
#if FIX == 2
    /* 7 threads keep calling a cheap callback (slot 6); time 10,000 register/unregister pairs of slot 7. */
    Game *g2 = (Game *)calloc(1, sizeof(Game));
    Game *g3 = (Game *)calloc(1, sizeof(Game));
    table[6] = (Slot){ 1, 17, { g2, 0 } };
    HANDLE bt[7];
    for (int k = 0; k < 7; k++) bt[k] = CreateThread(NULL, 0, busy_thread, (LPVOID)(((uintptr_t)1 << 32) | 6), 0, NULL);
    Sleep(50);
    LARGE_INTEGER t0, t1, f;
    QueryPerformanceFrequency(&f);
    QueryPerformanceCounter(&t0);
    for (int i = 0; i < 10000; i++) {
        table[7].env = (Env){ g3, 0 };
        __atomic_store_n(&table[7].type, 17u, __ATOMIC_RELAXED);
        unregister(7);
        g3->score = 0;
    }
    QueryPerformanceCounter(&t1);
    stop = 1;
    WaitForMultipleObjects(7, bt, TRUE, INFINITE);
    double us = (double)(t1.QuadPart - t0.QuadPart) * 1e6 / (double)f.QuadPart / 10000.0;
    fprintf(stderr, "unregister with 7 threads calling callbacks: %.3f us each\n", us);
    h_report("ffi-toctou-2-unregister", (uint64_t)(us * 1000.0));
#else
    h_report(FIX ? "ffi-toctou-1-fixed" : "ffi-toctou-0-as-proposed", (uint64_t)observed_poison);
#endif
    return 0;
}
