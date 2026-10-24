/* get.c: A5 / ODR-286. `reg.get[Pos](e).x += 1.0` through a table of type-erased pools.
   One file, three models chosen by -D:
     -DPLAIN : plain C (EnTT's static_cast: no type compare, no bounds checks; only the "pool exists" test)
     -DTYPED : Ember with a pool table keyed by the type itself (the slot for Pos can only hold a Pool[Pos],
               so the downcast needs no compare); Ember's bounds checks on sparse[e] and dense[i]
     -DCAST  : Ember with Map-like `Box[dyn AnyPool]` slots and `cast ... as Pool[Pos]` of the place:
               TYPED plus one compare of the pool's dynamic type id
   -DNOINLINE keeps get_pos out of line (a call per get), otherwise it is inlined into the loop. */
#include "harness.h"
#define NE (1u << 20)
#define NGET 32000000u
#define NTYPES 8
#define T_POS 3
#define POS_TYPE_ID 0x9e3779b9u
typedef struct { float x, y; } Pos;
typedef struct { uint32_t type_id; uint32_t n; uint32_t *sparse; Pos *dense; } Pool;
Pool *pools[NTYPES];          /* not static: other code could change it, as in a real program */

#ifdef NOINLINE
#define GETATTR __attribute__((noinline)) static
#else
#define GETATTR static inline
#endif
static void panic_(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }

GETATTR Pos *get_pos(uint32_t e) {
    Pool *p = pools[T_POS];
    if (!p) panic_("no Pos pool");
#if defined(CAST)
    if (p->type_id != POS_TYPE_ID) panic_("cast failed");
#endif
#if defined(TYPED) || defined(CAST)
    if (e >= p->n) panic_("index out of range");
    uint32_t i = p->sparse[e];
    if (i >= p->n) panic_("index out of range");
    return &p->dense[i];
#else
    return &p->dense[p->sparse[e]];
#endif
}

int main(void) {
    h_start(); h_seed(7);
    Pool *p = (Pool *)calloc(1, sizeof(Pool));
    p->type_id = POS_TYPE_ID; p->n = NE;
    p->sparse = (uint32_t *)malloc(NE * sizeof(uint32_t));
    p->dense = (Pos *)malloc(NE * sizeof(Pos));
    for (uint32_t i = 0; i < NE; i++) p->sparse[i] = i;
    for (uint32_t i = NE - 1; i > 0; i--) {             /* entities scattered over the dense array */
        uint32_t j = h_rand() % (i + 1), t = p->sparse[i]; p->sparse[i] = p->sparse[j]; p->sparse[j] = t;
    }
    for (uint32_t i = 0; i < NE; i++) { p->dense[i].x = 0.0f; p->dense[i].y = (float)i; }
    pools[T_POS] = p;
    __asm__ volatile("" ::: "memory");   /* setup is opaque to the loop: no folding of the type id or sizes */
    LARGE_INTEGER w0, w1, wf; QueryPerformanceCounter(&w0);
    uint64_t rng = h_rng;              /* h_rand's sequence in a local, so an out-of-line call does not force */
    for (uint32_t k = 0; k < NGET; k++) {  /* the global through memory (Rust's twin keeps it local too) */
        rng = rng * 6364136223846793005ull + 1442695040888963407ull;
        uint32_t e = (uint32_t)(rng >> 33) & (NE - 1);
        get_pos(e)->x += 1.0f;
    }
    QueryPerformanceCounter(&w1); QueryPerformanceFrequency(&wf);
    printf("work_ms=%.3f ", (double)(w1.QuadPart - w0.QuadPart) * 1000.0 / (double)wf.QuadPart);
    uint64_t sum = 0;
    for (uint32_t i = 0; i < NE; i++) sum += (uint64_t)p->dense[i].x * (i + 1);
#if defined(PLAIN)
    const char *name = "get-plain";
#elif defined(TYPED)
    const char *name = "get-typed";
#else
    const char *name = "get-cast";
#endif
    h_report(name, sum);
    return 0;
}
