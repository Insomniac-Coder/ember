/* ffi2.c: the checked trampoline (A20), with the fixes the attack needs (phase3b-threads-attack.md).
 *
 * Same workload as ../../bench/ffi.c: a C "library" `fire(cb, ud, n)`, not inlined, calls the callback
 * through a pointer it cannot see through, 200,000,000 times; the callback adds 1 to `game.score`.
 *
 *   -DMODEL=0  plain C: `ud` is the `Game*` (ffi.c model 0)
 *   -DMODEL=2  the checked trampoline as proposed (ffi.c model 2): range, generation, creator thread
 *   -DMODEL=5  model 2 plus a type check: the slot records which trampoline (closure type) registered it,
 *              and the trampoline compares it, so a user-data value from another callback cannot run this
 *              trampoline's code on that callback's state (type confusion)
 *   -DMODEL=6  a `threads(any)` callback, protected against being unregistered while a call is running on
 *              another thread by an atomic running count: one compare-exchange on {generation, running}
 *              to enter (it also checks the generation), one atomic decrement to leave
 *   -DMODEL=7  a `threads(any)` callback, protected by announcing the slot in the calling thread's own
 *              record (a small stack, for nested callbacks) with plain stores; unregistering, which is rare,
 *              pays a process-wide barrier (FlushProcessWriteBuffers) and waits until no record announces the
 *              slot (ffi_toctou.c shows the protocol working; ffi_unreg.c times the unregister)
 *   -DMODEL=8  model 5 written better: the slot's generation and type are one 64-bit word, compared with one
 *              instruction against {this trampoline's type, the handle's generation}; only on a mismatch does
 *              the trampoline look at which half differs (stale: return; another callback's: panic)
 *   -DMODEL=9  model 7 with model 8's single compare
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n ffi2.c -o bin/ffi2_n.exe */
#include "../../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#define CALLS 200000000

typedef void (*cb_t)(void *);
static __declspec(noinline) void fire(cb_t cb, void *ud, int n) {
    for (int i = 0; i < n; i++) cb(ud);
}
static __declspec(noinline) void fail(const char *what) { fprintf(stderr, "panic: %s\n", what); exit(101); }

#if MODEL == 0
typedef struct { int64_t score; } Game;
static void on_event(void *ud) { ((Game *)ud)->score += 1; }
#else
typedef struct { uint32_t strong, type_index; int64_t score; } Game;
typedef struct { Game *game; } Env;
static void lambda_body(Env *env) { env->game->score += 1; }
#define THIS_TRAMPOLINE 17u                          /* the id the compiler gives this closure type */

#if MODEL == 2 || MODEL == 5
typedef struct { uint32_t generation; uint32_t creator; uint32_t type; uint32_t pad; Env env; } Slot;   /* 24 bytes */
#elif MODEL == 8
typedef struct { uint32_t generation; uint32_t type; uint32_t creator; uint32_t pad; Env env; } Slot;   /* 24 bytes */
#elif MODEL == 9
typedef struct { uint32_t generation; uint32_t type; Env env; } Slot;   /* 16 bytes */
typedef struct { uint32_t depth, pending; uint64_t h[14]; } ThreadRec;
static __declspec(thread) ThreadRec *t_rec;
static __declspec(noinline) void slow_enter(void) { fail("callbacks nested 14 deep (would take the counted path)"); }
static __declspec(noinline) void run_pending(ThreadRec *r) { r->pending = 0; }
#elif MODEL == 6
typedef struct { uint64_t gen_running; uint32_t type; uint32_t pad; Env env; } Slot;   /* generation in the high half */
#else
typedef struct { uint32_t generation; uint32_t type; Env env; } Slot;   /* 16 bytes */
typedef struct { uint32_t depth, pending; uint64_t h[14]; } ThreadRec;   /* one per attached thread, 128 bytes */
static __declspec(thread) ThreadRec *t_rec;
static __declspec(noinline) void slow_enter(void) { fail("callbacks nested 14 deep (would take the counted path)"); }
static __declspec(noinline) void run_pending(ThreadRec *r) { r->pending = 0; }
#endif
static Slot *volatile table;
static uint32_t volatile table_len;
static __declspec(thread) uint32_t t_thread_id;

#if MODEL == 8 || MODEL == 9
/* Only on a mismatch: a stale handle returns quietly, another callback's handle panics. */
static __declspec(noinline) int mismatch(uint64_t word, uint64_t want) {
    if ((uint32_t)word != (uint32_t)want) return 0;
    fail("user data of another callback");
    return 0;
}
#endif
static void tramp(void *ud) {
    uintptr_t h = (uintptr_t)ud;
    if ((uint32_t)h >= table_len) fail("a callback handle C made up");
    Slot *s = &table[(uint32_t)h];
#if MODEL == 8
    uint64_t want = ((uint64_t)THIS_TRAMPOLINE << 32) | (uint32_t)(h >> 32);
    uint64_t word = __atomic_load_n((uint64_t *)&s->generation, __ATOMIC_ACQUIRE);
    if (word != want && !mismatch(word, want)) return;
    if (s->creator != t_thread_id) fail("callback on a thread other than its creator");
    lambda_body(&s->env);
#elif MODEL == 9
    ThreadRec *r = t_rec;
    uint32_t d = r->depth;
    if (d >= 14) slow_enter();
    __atomic_store_n(&r->h[d], (uint64_t)h, __ATOMIC_RELAXED);
    __atomic_store_n(&r->depth, d + 1, __ATOMIC_RELAXED);
    __atomic_signal_fence(__ATOMIC_SEQ_CST);
    uint64_t want = ((uint64_t)THIS_TRAMPOLINE << 32) | (uint32_t)(h >> 32);
    uint64_t word = __atomic_load_n((uint64_t *)&s->generation, __ATOMIC_RELAXED);
    if (word == want || mismatch(word, want)) lambda_body(&s->env);
    __atomic_store_n(&r->depth, d, __ATOMIC_RELEASE);
    if (r->pending) run_pending(r);
#elif MODEL == 2 || MODEL == 5
    if (__atomic_load_n(&s->generation, __ATOMIC_ACQUIRE) != (uint32_t)(h >> 32)) return;
#if MODEL == 5
    if (s->type != THIS_TRAMPOLINE) fail("user data of another callback");
#endif
    if (s->creator != t_thread_id) fail("callback on a thread other than its creator");
    lambda_body(&s->env);
#elif MODEL == 6
    uint64_t seen = __atomic_load_n(&s->gen_running, __ATOMIC_RELAXED);
    for (;;) {
        if ((uint32_t)(seen >> 32) != (uint32_t)(h >> 32)) return;   /* unregistered */
        if (__atomic_compare_exchange_n(&s->gen_running, &seen, seen + 1, 1, __ATOMIC_ACQUIRE, __ATOMIC_RELAXED)) break;
    }
    if (s->type != THIS_TRAMPOLINE) fail("user data of another callback");
    lambda_body(&s->env);
    __atomic_fetch_sub(&s->gen_running, 1, __ATOMIC_RELEASE);        /* unregister frees at running 0 */
#else
    ThreadRec *r = t_rec;
    uint32_t d = r->depth;
    if (d >= 14) slow_enter();
    __atomic_store_n(&r->h[d], (uint64_t)h, __ATOMIC_RELAXED);        /* announce: a plain store */
    __atomic_store_n(&r->depth, d + 1, __ATOMIC_RELAXED);
    __atomic_signal_fence(__ATOMIC_SEQ_CST);                           /* the light half: compiler order only */
    if (__atomic_load_n(&s->generation, __ATOMIC_RELAXED) == (uint32_t)(h >> 32)) {
        if (s->type != THIS_TRAMPOLINE) fail("user data of another callback");
        lambda_body(&s->env);
    }
    __atomic_store_n(&r->depth, d, __ATOMIC_RELEASE);                  /* withdraw */
    if (r->pending) run_pending(r);                                     /* a free this thread deferred */
#endif
}
#endif

static cb_t volatile the_cb;
static void *volatile the_ud;

int main(void) {
    h_start();
#if MODEL == 0
    Game *g = (Game *)calloc(1, sizeof(Game));
    the_cb = on_event; the_ud = g;
#else
    Game *g = (Game *)calloc(1, sizeof(Game)); g->strong = 2; g->type_index = 3;
    table = (Slot *)calloc(64, sizeof(Slot)); table_len = 64;
    t_thread_id = 1;
#if MODEL == 7 || MODEL == 9
    t_rec = (ThreadRec *)calloc(1, sizeof(ThreadRec));
#endif
    uint32_t slot = 5, gen = 9;
#if MODEL == 6
    table[slot].gen_running = (uint64_t)gen << 32;
#else
    table[slot].generation = gen;
#endif
#if MODEL == 2 || MODEL == 5 || MODEL == 8
    table[slot].creator = t_thread_id;
#endif
    table[slot].type = THIS_TRAMPOLINE; table[slot].env.game = g;
    the_cb = tramp; the_ud = (void *)(((uintptr_t)gen << 32) | slot);
#endif
    fire(the_cb, the_ud, CALLS);
    static const char *names[] = {"ffi2-0-plainC", "", "ffi2-2-checked", "", "", "ffi2-5-checked-typed",
                                  "ffi2-6-any-atomic-running", "ffi2-7-any-announced",
                                  "ffi2-8-checked-typed-one-compare", "ffi2-9-any-announced-one-compare"};
    h_report(names[MODEL], (uint64_t)g->score);
    return 0;
}
