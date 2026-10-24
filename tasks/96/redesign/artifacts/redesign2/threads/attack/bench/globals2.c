/* globals2.c: what a module variable whose initialiser runs on first access costs (A19).
 *
 * Same workload as ../../bench/globals.c (tick() not inlined, 100,000,000 calls: FRAME += 1, LOG.push(FRAME),
 * the log summed and cleared every 4096 pushes). The proposal says module variables compile to plain C
 * globals. That holds when the initialiser is evaluated at compile time (`0`, `[]`). For any other initialiser
 * (`CONFIG = load_config()`, `START = time.now()`), [STA-3] runs it on first access, and today's C backend
 * reaches such a static through an accessor that calls the runtime's out-of-line `ember_rt_static_begin`
 * on every access (compiler/ember_codegen_c/src/lib.rs `lazy_static_definitions`).
 *
 *   -DMODEL=0  plain C globals, initialised before main (globals.c model 0; also the eager-initialisation fix)
 *   -DMODEL=4  plain globals reached through a base pointer loaded at the start of tick() (globals.c model 4):
 *              the same addressing as model 6's accessor results, so 4 against 6 is the accessor's own cost
 *              without the CPU's store-to-load effect that ripchain.c isolates
 *   -DMODEL=6  each access goes through the lazy accessor: an out-of-line begin call that checks the state with
 *              an acquire load (three accesses per tick, as each expression naming the static calls it)
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n globals2.c -o bin/globals2_n.exe */
#include "../../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#define CALLS 100000000u

typedef struct { int64_t *p; size_t len, cap; } Vec;
static __declspec(noinline) void vec_grow(Vec *v) {
    size_t cap = v->cap ? v->cap * 2 : 4;
    int64_t *p = (int64_t *)realloc(v->p, cap * sizeof(int64_t));
    if (!p) abort();
    v->p = p; v->cap = cap;
}

#if MODEL == 0
static int64_t FRAME_storage;
static Vec LOG_storage;
#define FRAME FRAME_storage
#define LOG LOG_storage
#elif MODEL == 4
typedef struct { int64_t frame; Vec log; } ModVars;
static ModVars MODVARS;
static ModVars *volatile modvars_base = &MODVARS;
#define FRAME (mb->frame)
#define LOG (mb->log)
#else
/* The runtime's begin: 1 means "run the initialiser now". Out of line, as ember_rt_static_begin is. */
static __declspec(noinline) int rt_static_begin(uint32_t *state, const char *name) {
    (void)name;
    if (__atomic_load_n(state, __ATOMIC_ACQUIRE) == 2) return 0;
    *state = 1;
    return 1;
}
static __declspec(noinline) void rt_static_end(uint32_t *state) { __atomic_store_n(state, 2, __ATOMIC_RELEASE); }
static int64_t frame_init(void) { return 0; }
static Vec log_init(void) { Vec v = {0}; return v; }
static int64_t *FRAME_static(void) {
    static int64_t storage; static uint32_t state;
    if (rt_static_begin(&state, "FRAME")) { storage = frame_init(); rt_static_end(&state); }
    return &storage;
}
static Vec *LOG_static(void) {
    static Vec storage; static uint32_t state;
    if (rt_static_begin(&state, "LOG")) { storage = log_init(); rt_static_end(&state); }
    return &storage;
}
#define FRAME (*FRAME_static())
#define LOG (*LOG_static())
#endif

static uint64_t checksum;

static __declspec(noinline) void tick(void) {
#if MODEL == 4
    ModVars *mb = modvars_base;
#endif
    FRAME += 1;
    Vec *log = &LOG;
    int64_t n = FRAME;
    if (log->len == log->cap) vec_grow(log);
    log->p[log->len++] = n;
    if (log->len == 4096) {
        for (size_t i = 0; i < log->len; i++) checksum += (uint64_t)log->p[i];
        log->len = 0;
    }
}

int main(void) {
    h_start();
    for (uint32_t i = 0; i < CALLS; i++) tick();
    static const char *names[] = {"globals2-0-plain-eager", "", "", "", "globals2-4-plain-base-pointer", "", "globals2-6-lazy-accessor"};
    h_report(names[MODEL], checksum);
    return 0;
}
