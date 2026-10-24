/* globals.c: a module's frame counter and log, changed by a system function every call (A19).
 *
 * `tick()` (not inlined, as a system function in another module would not be) runs 100 million
 * times: `FRAME += 1` and `LOG.push(FRAME)`; every 4096 pushes the log is summed into the checksum and
 * cleared (Python: `LOG.clear()`), so memory stays bounded. One thread. Same checksum in every model.
 *
 *   -DMODEL=0  plain C globals. This is also exactly what the proposed module `var` compiles to (a
 *              global at a fixed address; the loan and thread rules are compile-time), so there is no
 *              separate model.
 *   -DMODEL=1  today's Ember workaround (phase 1's a19_workaround.em): `static FRAME: Atomic[int]` with
 *              `fetch_add` (sequentially consistent, `[THR-14]`) and `static LOG: Mutex[Array[int]]` locked
 *              per call, with the runtime's one-thread shortcut (`ember_mutex_try_lock` takes a plain write
 *              while `ember_threads_started` is 0, runtime/ember_rt/include/ember_rt.h)
 *   -DMODEL=2  as 1, after a second thread has started: the lock is an atomic compare-exchange and an
 *              exchange (the same header's multi-thread path)
 *   -DMODEL=3  thread-local globals (`__declspec(thread)`): every thread its own FRAME and LOG
 *   -DMODEL=4  plain globals, but the module's variables are one struct reached through a base pointer
 *              loaded once per call (model 3's addressing without the thread-local lookup), to test
 *              whether model 0 against model 3 is about the addressing mode
 *   -DMODEL=5  the module's variables as one struct global, named directly (clang addresses it from the
 *              instruction pointer, as model 0)
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n globals.c -o globals_n.exe */
#include "../../../redesign/bench/harness.h"

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
static int64_t FRAME;
static Vec LOG;
#elif MODEL == 5
typedef struct { int64_t frame; Vec log; } ModVars5;
static ModVars5 MV;
#define FRAME (MV.frame)
#define LOG (MV.log)
#elif MODEL == 4
typedef struct { int64_t frame; Vec log; } ModVars;
static ModVars MODVARS;
static ModVars *volatile modvars_base = &MODVARS;
#define FRAME (mb->frame)
#define LOG (mb->log)
#elif MODEL == 3
static __declspec(thread) int64_t FRAME;
static __declspec(thread) Vec LOG;
#else
static volatile int64_t FRAME;                 /* Atomic[int] */
static struct { uint32_t word; Vec v; } LOG;   /* Mutex[Array[int]]: the lock word, then the value */
static uint32_t ember_threads_started = (MODEL == 2);
static __declspec(noinline) void lock_contended(uint32_t *w) {
    uint32_t z;
    do { z = 0; YieldProcessor(); } while (!__atomic_compare_exchange_n(w, &z, 1u, 0, __ATOMIC_ACQUIRE, __ATOMIC_RELAXED));
}
static inline void mutex_lock(uint32_t *w) {
    if (__atomic_load_n(&ember_threads_started, __ATOMIC_RELAXED) == 0) {
        if (*w == 0) { *w = 1; return; }
        lock_contended(w);
        return;
    }
    uint32_t z = 0;
    if (!__atomic_compare_exchange_n(w, &z, 1u, 0, __ATOMIC_ACQUIRE, __ATOMIC_RELAXED)) lock_contended(w);
}
static inline void mutex_unlock(uint32_t *w) {
    if (__atomic_load_n(&ember_threads_started, __ATOMIC_RELAXED) == 0) { *w = 0; return; }
    __atomic_exchange_n(w, 0u, __ATOMIC_RELEASE);
}
#endif

static uint64_t checksum;

static __declspec(noinline) void tick(void) {
#if MODEL == 4
    ModVars *mb = modvars_base;
#endif
#if MODEL == 0 || MODEL == 3 || MODEL == 4 || MODEL == 5
    FRAME += 1;
    Vec *log = &LOG;
    int64_t n = FRAME;
#else
    int64_t n = __atomic_add_fetch(&FRAME, 1, __ATOMIC_SEQ_CST);
    mutex_lock(&LOG.word);
    Vec *log = &LOG.v;
#endif
    if (log->len == log->cap) vec_grow(log);
    log->p[log->len++] = n;
    if (log->len == 4096) {
        for (size_t i = 0; i < log->len; i++) checksum += (uint64_t)log->p[i];
        log->len = 0;
    }
#if MODEL == 1 || MODEL == 2
    mutex_unlock(&LOG.word);
#endif
}

int main(void) {
    h_start();
    for (uint32_t i = 0; i < CALLS; i++) tick();
    static const char *names[] = {"globals-0-plainC-and-var", "globals-1-atomic-mutex-1thread",
                                  "globals-2-atomic-mutex-threads", "globals-3-threadlocal", "globals-4-plain-base-pointer", "globals-5-struct-global"};
    h_report(names[MODEL], checksum);
    return 0;
}
