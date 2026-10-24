/* ffi.c: a C library calls an Ember callback 200 million times (A20).
 *
 * The "library" is `fire(cb, ud, n)`, not inlined, calling `cb(ud)` n times through a function pointer
 * it receives (the pointer is read from a volatile, so the C compiler cannot see which function it is).
 * The callback adds 1 to `game.score` (Python: `lambda ud: setattr(game, "score", game.score + 1)`).
 *
 *   -DMODEL=0  plain C: `ud` is the `Game*`; the callback is `((Game*)ud)->score++`. Rust's workaround
 *              (`&mut *(ud as *mut Game)`, phase 1 a20_ffi.rs) compiles to the same thing.
 *   -DMODEL=1  today's Ember (`[FFI-21]`, the trampoline written by `emit_ffi_callback`,
 *              compiler/ember_codegen_c/src/lib.rs): an out-of-line `ember_rt_thread_attach()` (a
 *              thread-local generation compared with a global one, runtime/ember_rt/src/ember_rt.c
 *              `runtime_thread_attached`), then the record at `ud`, then the lambda's body with the record's
 *              environment, which holds the captured `game` handle (a lean class object: 8-byte header)
 *   -DMODEL=2  the proposed checked trampoline: `ud` is not an address but {slot, generation} in a table
 *              of registered callbacks; the trampoline checks the slot is inside the table (C may hand back
 *              any value), loads the slot, compares the generation (a call after unregistering finds a
 *              different generation and does nothing), compares the calling thread with the slot's creator
 *              thread (a thread-local id; this also proves the thread is attached, so the out-of-line attach
 *              call goes), then runs the body. No running count: the body's effect summary shows it drops
 *              no callback token, so it cannot unregister itself while it runs
 *   -DMODEL=4  model 2 for a body that may drop a callback token (its own): the trampoline counts the call
 *              as running around the body, and unregistering frees the state only at a count of 0
 *   -DMODEL=3  model 2 plus model 1's out-of-line attach call (if the attach could not be folded in)
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n ffi.c -o ffi_n.exe */
#include "../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#define CALLS 200000000

typedef void (*cb_t)(void *);

/* The foreign library: it only ever sees a function pointer and a void*. */
static __declspec(noinline) void fire(cb_t cb, void *ud, int n) {
    for (int i = 0; i < n; i++) cb(ud);
}

static __declspec(noinline) void fail(const char *what) { fprintf(stderr, "panic: %s\n", what); exit(101); }

#if MODEL == 0
typedef struct { int64_t score; } Game;
static void on_event(void *ud) { ((Game *)ud)->score += 1; }
#else
typedef struct { uint32_t strong, type_index; int64_t score; } Game;   /* a lean class object */
typedef struct { Game *game; } Env;                                    /* the lambda's captures */
static void lambda_body(Env *env) { env->game->score += 1; }

/* The runtime's attach check, out of line as in ember_rt.c. */
static uint32_t g_thread_generation = 1;
static __declspec(thread) uint32_t g_attached_generation;
static __declspec(noinline) void rt_thread_attach(void) {
    if (g_attached_generation != __atomic_load_n(&g_thread_generation, __ATOMIC_RELAXED))
        g_attached_generation = __atomic_load_n(&g_thread_generation, __ATOMIC_RELAXED);
}
#endif

#if MODEL == 1
typedef struct { Env env; } Record;                                    /* `retained`: the boxed closure */
static void tramp(void *ud) {
    rt_thread_attach();
    Record *r = (Record *)ud;
    lambda_body(&r->env);
}
#elif MODEL == 2 || MODEL == 3 || MODEL == 4
typedef struct { uint32_t generation; uint32_t creator; uint32_t running; uint32_t pad; Env env; } Slot;  /* 24 bytes */
static Slot *volatile table;    /* the registered callbacks; volatile so clang cannot turn the table into a static array, which a growable runtime table is not */
static uint32_t volatile table_len;
static __declspec(thread) uint32_t t_thread_id;                       /* set when a thread attaches */
static void tramp(void *ud) {
#if MODEL == 3
    rt_thread_attach();
#endif
    uintptr_t h = (uintptr_t)ud;
    if ((uint32_t)h >= table_len) fail("a callback handle C made up");
    Slot *s = &table[(uint32_t)h];
    if (s->generation != (uint32_t)(h >> 32)) return;                  /* unregistered: nothing runs */
    if (s->creator != t_thread_id) fail("callback on a thread other than its creator");
#if MODEL == 4
    s->running += 1;                                                   /* plain: only the creator thread calls */
#endif
    lambda_body(&s->env);
#if MODEL == 4
    s->running -= 1;                                                   /* unregister frees the state when 0 */
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
#elif MODEL == 1
    Game *g = (Game *)calloc(1, sizeof(Game)); g->strong = 2; g->type_index = 3;
    Record *r = (Record *)malloc(sizeof(Record)); r->env.game = g;    /* registration boxes the closure */
    the_cb = tramp; the_ud = r;
#else
    Game *g = (Game *)calloc(1, sizeof(Game)); g->strong = 2; g->type_index = 3;
    table = (Slot *)calloc(64, sizeof(Slot)); table_len = 64;
    t_thread_id = 1;                                                   /* the main thread attaches */
    uint32_t slot = 5, gen = 9;
    table[slot].generation = gen; table[slot].creator = t_thread_id; table[slot].env.game = g;
    the_cb = tramp; the_ud = (void *)(((uintptr_t)gen << 32) | slot);
#endif
    fire(the_cb, the_ud, CALLS);
    static const char *names[] = {"ffi-0-plainC", "ffi-1-today-trampoline", "ffi-2-checked-trampoline",
                                  "ffi-3-checked-plus-attach", "ffi-4-checked-running-count"};
    h_report(names[MODEL], (uint64_t)g->score);
    return 0;
}
