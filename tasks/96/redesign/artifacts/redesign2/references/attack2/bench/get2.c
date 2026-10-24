/* get2.c: A5 (answers report 3.4), `reg.get[Pos](e).x += 1.0`, with the data laid out as Ember lays it out.
   The answers report's get.c used one length `n` for both bounds checks, kept the type id inside the pool,
   and stored a plain pointer per slot. Here:
     - every Array is Ember's header {ptr, len, cap} (runtime/ember_rt/include/ember_rt.h, `ember_vec`), so
       `sparse[e]` checks sparse.len and `dense[i]` checks dense.len: two lengths, two loads;
     - a slot is a `Box[dyn AnyPool]`: {data pointer, method-table pointer}, 16 bytes, NULL data = empty;
     - the type id lives in the method table (one per type, never in the pool), as a 64-bit number.
   Models, one -D each:
     -DTABLE  : the pool table keyed by the type: slot number known when compiled, no type check.
     -DTABLE -DSLOTVAR : the same, with the slot number loaded at run time and checked against the table
                length (separate packages, or a hot-reload build where numbers must stay stable).
     -DTABLE -DSLOTHOIST : the saved version: the run-time slot number is read and checked once, before the loop
                (a number the runtime assigns never changes afterwards, so the generated code may keep it in a
                local; the per-get work is then TABLE's).
     -DCAST   : `cast self.pools[k] as Pool[Pos]` of a Box[dyn] place: TABLE plus a load of the method
                table pointer, a load of its type id, and a compare.
   Plain C and the answers report's own models are its get.c, built unchanged. */
#include "harness.h"
#define NE (1u << 20)
#define NGET 32000000u
#define NTYPES 8
#define T_POS 3
#define POS_TYPE_ID 0x7f3a9c5e12d40b61ull
typedef struct { float x, y; } Pos;
typedef struct { void *ptr; size_t len, cap; } Vec;                 /* ember_vec */
typedef struct { Vec sparse; Vec dense; } Pool;                     /* Pool[Pos]: sparse: Array[u32], dense: Array[Pos] */
typedef struct { void (*drop)(void *); uint64_t type_id; int64_t (*count)(void *); } VTable;
typedef struct { void *data; const VTable *vt; } DynBox;            /* Box[dyn AnyPool] */
DynBox pools[NTYPES];                       /* not static: other code could change it, as in a real program */
#if defined(SLOTVAR) || defined(SLOTHOIST)
uint32_t pos_slot = T_POS;                  /* not static, not const: assigned when the program is loaded */
uint32_t n_slots = NTYPES;
#endif
static void pool_drop(void *p) { Pool *q = (Pool *)p; free(q->sparse.ptr); free(q->dense.ptr); free(q); }
static int64_t pool_count(void *p) { return (int64_t)((Pool *)p)->dense.len; }
const VTable POOL_POS_VT = { pool_drop, POS_TYPE_ID, pool_count };
static void panic_(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }

static inline Pos *get_pos(uint32_t e, uint32_t slot) {
#if defined(SLOTVAR)
    (void)slot;
    if (pos_slot >= n_slots) panic_("no Pos slot");
    DynBox *s = &pools[pos_slot];
#elif defined(SLOTHOIST)
    DynBox *s = &pools[slot];
#else
    (void)slot;
    DynBox *s = &pools[T_POS];
#endif
    if (!s->data) panic_("no Pos pool");
#ifdef CAST
    if (s->vt->type_id != POS_TYPE_ID) panic_("cast failed");
#endif
    Pool *p = (Pool *)s->data;
    if ((size_t)e >= p->sparse.len) panic_("index out of range");
    uint32_t i = ((uint32_t *)p->sparse.ptr)[e];
    if ((size_t)i >= p->dense.len) panic_("index out of range");
    return &((Pos *)p->dense.ptr)[i];
}

int main(void) {
    h_start(); h_seed(7);
    Pool *p = (Pool *)calloc(1, sizeof(Pool));
    uint32_t *sparse = (uint32_t *)malloc(NE * sizeof(uint32_t));
    Pos *dense = (Pos *)malloc(NE * sizeof(Pos));
    for (uint32_t i = 0; i < NE; i++) sparse[i] = i;
    for (uint32_t i = NE - 1; i > 0; i--) {             /* the same shuffle as get.c */
        uint32_t j = h_rand() % (i + 1), t = sparse[i]; sparse[i] = sparse[j]; sparse[j] = t;
    }
    for (uint32_t i = 0; i < NE; i++) { dense[i].x = 0.0f; dense[i].y = (float)i; }
    p->sparse = (Vec){ sparse, NE, NE };
    p->dense = (Vec){ dense, NE, NE };
    pools[T_POS] = (DynBox){ p, &POOL_POS_VT };
    __asm__ volatile("" ::: "memory");
    LARGE_INTEGER w0, w1, wf; QueryPerformanceCounter(&w0);
    uint64_t rng = h_rng;
#ifdef SLOTHOIST
    uint32_t slot = pos_slot;                          /* read and checked once per function */
    if (slot >= n_slots) panic_("no Pos slot");
#else
    uint32_t slot = T_POS;
#endif
    for (uint32_t k = 0; k < NGET; k++) {
        rng = rng * 6364136223846793005ull + 1442695040888963407ull;
        uint32_t e = (uint32_t)(rng >> 33) & (NE - 1);
        get_pos(e, slot)->x += 1.0f;
    }
    QueryPerformanceCounter(&w1); QueryPerformanceFrequency(&wf);
    printf("work_ms=%.3f ", (double)(w1.QuadPart - w0.QuadPart) * 1000.0 / (double)wf.QuadPart);
    uint64_t sum = 0;
    for (uint32_t i = 0; i < NE; i++) sum += (uint64_t)dense[i].x * (i + 1);
#if defined(CAST)
    const char *name = "get2-cast";
#elif defined(SLOTVAR)
    const char *name = "get2-table-slotvar";
#elif defined(SLOTHOIST)
    const char *name = "get2-table-slothoist";
#else
    const char *name = "get2-table";
#endif
    h_report(name, sum);
    pools[T_POS].vt->drop(pools[T_POS].data);
    return 0;
}
