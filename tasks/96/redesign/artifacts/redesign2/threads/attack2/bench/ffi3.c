/* ffi3.c: the checked trampoline (A20) as the first attack saved it (its ffi2.c models 0, 8, 9, copied
 * unchanged), plus the user-data value made unusable as an address (this attack, section 7.2).
 *
 * Same workload as ../../bench/ffi.c and ../../attack/bench/ffi2.c: a C "library" `fire(cb, ud, n)`, not
 * inlined, calls the callback through a pointer it cannot see through, 200,000,000 times; the callback adds
 * 1 to `game.score`.
 *
 *   -DMODEL=0   plain C: `ud` is the `Game*`
 *   -DMODEL=8   the first attack's saved trampoline: range check, then {generation, closure type} in one
 *               64-bit compare, then the creator thread
 *   -DMODEL=9   the first attack's saved `threads(any)` trampoline: announce the slot (two plain stores),
 *               the one compare, withdraw
 *   -DMODEL=10  model 8 with a tagged user data: the high 16 bits of the value C holds are 0x4080 (bit 62 and
 *               bit 55 set, bit 63 clear), so the value is not a canonical x86-64 address (and, with bit 55
 *               set, an AArch64 kernel-half address): C code that reads or writes through it faults at once.
 *               The tag is stored in the slot as part of the generation word, so the trampoline's compare is
 *               the same instruction against a different constant: no instruction is added.
 *   -DMODEL=11  model 9 with the same tag
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n ffi3.c -o bin/ffi3_n.exe */
#include "../../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#define CALLS 200000000
#if MODEL == 10 || MODEL == 11
#define TAG 0x40800000u            /* the generation word's top 16 bits: value bits 62 and 55 */
#else
#define TAG 0u
#endif

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
#define THIS_TRAMPOLINE 17u

#if MODEL == 8 || MODEL == 10
typedef struct { uint32_t generation; uint32_t type; uint32_t creator; uint32_t pad; Env env; } Slot;   /* 24 bytes */
#else
typedef struct { uint32_t generation; uint32_t type; Env env; } Slot;   /* 16 bytes */
typedef struct { uint32_t depth, pending; uint64_t h[14]; } ThreadRec;
static __declspec(thread) ThreadRec *t_rec;
static __declspec(noinline) void slow_enter(void) { fail("callbacks nested 14 deep (would take the counted path)"); }
static __declspec(noinline) void run_pending(ThreadRec *r) { r->pending = 0; }
#endif
static Slot *volatile table;
static uint32_t volatile table_len;
static __declspec(thread) uint32_t t_thread_id;

static __declspec(noinline) int mismatch(uint64_t word, uint64_t want) {
    if ((uint32_t)word != (uint32_t)want) return 0;
    fail("user data of another callback");
    return 0;
}
static void tramp(void *ud) {
    uintptr_t h = (uintptr_t)ud;
    if ((uint32_t)h >= table_len) fail("a callback handle C made up");
    Slot *s = &table[(uint32_t)h];
#if MODEL == 8 || MODEL == 10
    uint64_t want = ((uint64_t)THIS_TRAMPOLINE << 32) | (uint32_t)(h >> 32);
    uint64_t word = __atomic_load_n((uint64_t *)&s->generation, __ATOMIC_ACQUIRE);
    if (word != want && !mismatch(word, want)) return;
    if (s->creator != t_thread_id) fail("callback on a thread other than its creator");
    lambda_body(&s->env);
#else
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
#if MODEL == 9 || MODEL == 11
    t_rec = (ThreadRec *)calloc(1, sizeof(ThreadRec));
#endif
    uint32_t slot = 5, gen = TAG | 9u;           /* with the tag, 16 bits of generation remain */
    table[slot].generation = gen;
#if MODEL == 8 || MODEL == 10
    table[slot].creator = t_thread_id;
#endif
    table[slot].type = THIS_TRAMPOLINE; table[slot].env.game = g;
    the_cb = tramp; the_ud = (void *)(((uintptr_t)gen << 32) | slot);
#endif
    fire(the_cb, the_ud, CALLS);
    static const char *names[] = {"ffi3-0-plainC", "", "", "", "", "", "", "", "ffi3-8-checked-typed-one-compare",
                                  "ffi3-9-any-announced-one-compare", "ffi3-10-checked-tagged", "ffi3-11-any-announced-tagged"};
    h_report(names[MODEL], (uint64_t)g->score);
    return 0;
}
