/* cb_rt.h: the checked trampoline's runtime ([FFI-53], [FFI-22], [FFI-33a], [FFI-33b]; R12, R15),
   as the cb_* models and salvages use it. One mode macro picks the thread check:

     (none)            the rule as written for a `threads(creator)` callback in the release profile:
                       attach ([FFI-22], a thread-local compare), the slot inside the table, the
                       generation and the function's type in one compare, then the calling thread
                       against the slot's creator ([FFI-53]: "in every profile").
     NO_THREAD_CHECK   the same without the thread compare: today's release profile, where the check
                       runs only in debug (tests/conformance/FFI-33b). The "without" of the cost row.
     FOLD_ATTACH       salvage: the thread compare also stands for the attach check (see below).
     TEB_THREAD        salvage: the creator is the OS thread id, compared with the one the thread's own
                       environment block holds (one load from gs), standing for attach too.

   C holds a tagged number, never an address ([FFI-53]): the slot in the low 48 bits, the slot's
   generation in the top 16, never all zero (so the number is no user-space address). */
#pragma once
#include "m4.h"
#if defined(TEB_THREAD)
#include <intrin.h>
#endif

#define SLOT_MASK 0xFFFFFFFFFFFFull
#define USED_BIT (1ull << 48)          /* salvage ONCE_GENMARK: the used-up mark inside the compared word */
#define F_ONCE 1u                      /* a `callback(once)` slot */
#define F_USED 2u                      /* the model's used-up mark ([FFI-53]) */
#define F_PREV_ONCE 4u                 /* the previous generation ended by its one call (message only) */

/* [FFI-22] attaching, [FFI-33a] thread numbers. Non-static globals: the compiler must load them, as it
   loads the runtime's from ember_rt.c. */
uint32_t g_thread_generation = 1;
uint32_t g_thread_numbers;
static TLS uint32_t t_attached_generation;
static TLS uint32_t t_thread_number;
static NOINLINE void rt_attach_slow(void) {
    t_attached_generation = g_thread_generation;
    if (t_thread_number == 0) t_thread_number = ++g_thread_numbers;
}
static FORCEINLINE void rt_attach(void) {
    if (UNLIKELY(t_attached_generation != g_thread_generation)) rt_attach_slow();
}
static FORCEINLINE uint32_t rt_thread_id(void) {
#if defined(TEB_THREAD)
    return __readgsdword(0x48);        /* TEB ClientId.UniqueThread: what GetCurrentThreadId returns */
#else
    return t_thread_number;
#endif
}

/* The table of registered callbacks: reserved once, never moves; a slot is 32 bytes. word = generation
   (16 bits, never 0) | the trampoline's type id << 16. env is the function value's state, in the slot. */
typedef struct { uint64_t word; uint32_t creator; uint32_t flags; void *env[2]; } Slot;
Slot *g_table;
uint64_t g_table_len;
static uint64_t g_table_cap;
static uint32_t *g_free;
static uint32_t g_free_n;

static void cb_init(uint64_t cap) {
    g_table = (Slot *)calloc((size_t)cap, sizeof(Slot));
    g_free = (uint32_t *)malloc((size_t)cap * sizeof(uint32_t));
    if (!g_table || !g_free) e_panic("out of memory");
    g_table_cap = cap; g_table_len = 0; g_free_n = 0;
    rt_attach();
}

/* Registration: a free slot (or a new one), its generation, its type, its creator, its state. */
static uint64_t cb_register(uint32_t type, uint32_t flags, void *e0, void *e1) {
    uint64_t slot;
    if (g_free_n) slot = g_free[--g_free_n];
    else {
        if (g_table_len == g_table_cap) e_panic("too many callbacks");
        slot = g_table_len++;
    }
    Slot *s = &g_table[slot];
    uint64_t gen = s->word & 0xFFFF;
    if (gen == 0) gen = 1;
    s->word = gen | ((uint64_t)type << 16);
    s->creator = rt_thread_id();
    s->flags = (s->flags & F_PREV_ONCE) | flags;
    s->env[0] = e0; s->env[1] = e1;
    return (gen << 48) | slot;
}

/* The end of a registration (a token ended, a borrowed call returned, a once callback ran): the next
   generation, so every number C still holds is stale; at the last generation the slot is retired
   ([FFI-53]: one compare per registration, nothing per call). */
static void cb_end(uint64_t slot) {
    Slot *s = &g_table[slot];
    uint64_t gen = s->word & 0xFFFF;
    uint32_t prev = (s->flags & F_ONCE) ? F_PREV_ONCE : 0;
    s->flags = prev;
    s->env[0] = s->env[1] = 0;
    if (gen == 0xFFFF) { s->word = 0; return; }        /* retired: never on the free list again */
    s->word = gen + 1;
    g_free[g_free_n++] = (uint32_t)slot;
}

/* The stops ([FFI-53]'s texts; the line numbers are the example's). Cold, out of line. */
static NORETURN NOINLINE void cb_stop(uint64_t h, uint32_t type) {
    uint64_t slot = h & SLOT_MASK;
    if (slot >= g_table_len) e_panic("C called a callback with a number no registration gave");
    Slot *s = &g_table[slot];
    uint64_t gen = h >> 48, now = s->word & 0xFFFF;
    if ((s->word & USED_BIT) && gen == now)
        e_panic("C called the callback registered on line 29 a second time; it was registered `callback(once)`, so it may run only once");
    if (gen != now) {
        if ((s->flags & F_PREV_ONCE) && gen + 1 == now)
            e_panic("C called the callback registered on line 29 a second time; it was registered `callback(once)`, so it may run only once");
        e_panic("C called the callback registered on line 16 after its token ended; it was unregistered");
    }
    (void)type;
    e_panic("C called a callback with the number of another callback");
}
static NORETURN NOINLINE void cb_stop_thread(void) {
    e_panic("C called the callback registered on line 16 on another thread, but its contract says `threads(creator)`");
}
static NORETURN NOINLINE void cb_stop_once(void) {
    e_panic("C called the callback registered on line 29 a second time; it was registered `callback(once)`, so it may run only once");
}
#if defined(FOLD_ATTACH) || defined(TEB_THREAD)
/* The thread compare failed: attach (a thread that detached, or a runtime started again, has no
   number / its slots were ended), then compare again; still different: another thread. */
static NOINLINE void cb_thread_slow(Slot *s) {
    rt_attach_slow();
    if (s->creator != rt_thread_id()) cb_stop_thread();
}
#endif

/* Every trampoline's entry: returns the slot whose state the body uses. */
static FORCEINLINE Slot *cb_enter(void *ud, uint32_t type) {
#if !defined(FOLD_ATTACH) && !defined(TEB_THREAD)
    rt_attach();                                                /* [FFI-22] */
#endif
    uint64_t h = (uint64_t)(uintptr_t)ud;
    uint64_t slot = h & SLOT_MASK;
    if (UNLIKELY(slot >= g_table_len)) cb_stop(h, type);        /* [FFI-53] inside the table */
    Slot *s = &g_table[slot];
    uint64_t want = (h >> 48) | ((uint64_t)type << 16);
    if (UNLIKELY(s->word != want)) cb_stop(h, type);            /* [FFI-53] generation and type, one compare */
#if defined(FOLD_ATTACH) || defined(TEB_THREAD)
    if (UNLIKELY(s->creator != rt_thread_id())) cb_thread_slow(s);   /* thread, standing for attach too */
#elif !defined(NO_THREAD_CHECK)
    if (UNLIKELY(s->creator != rt_thread_id())) cb_stop_thread();    /* [FFI-53], [FFI-33b]: every profile */
#endif
    return s;
}
