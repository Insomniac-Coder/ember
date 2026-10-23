/* phase 2 (Vale / Austral / Pony): what a dereference costs against plain C.
   raw     : a plain C pointer (Austral and Vale's owning references compile to this)
   genref  : Vale's generational reference: the reference is {pointer, remembered generation},
             the object carries a generation word; every dereference compares them and halts on
             a mismatch (verdagon.dev/blog/generational-references, "random generations" variant)
   fat     : genref's layout (16-byte reference, generation word in the object) with the check
             removed: separates the cost of the bigger layout from the cost of the compare
   rc      : naive non-atomic reference counting: taking a reference into a local adds 1 to the
             object's count, dropping it subtracts 1 and frees at 0 (the "RC" row of Vale's own
             table; also what a Swift/Python-style handle does)
   Two access patterns over N separately malloc'd 16-byte objects:
     *_seq  : walk the reference array in order (objects mostly in allocation order: cache-friendly)
     *_rand : 16M random references (memory-bound)
   One mode per process so peak_kb belongs to that mode; work_ms times the phase after the fill.
   Build: clang -O2 genref.c -o genref.exe   Run: genref.exe <mode>  (pins itself to logical CPU 1) */
#include "../bench/harness.h"
#include <string.h>

#define N      (1u << 22)   /* 4,194,304 objects */
#define PASSES 20
#define HITS   (1u << 24)

typedef struct { int64_t a, b; } Obj;                       /* raw: 16 bytes */
typedef struct { uint64_t gen; int64_t a, b; } GObj;        /* genref: + 8-byte generation */
typedef struct { uint64_t rc; int64_t a, b; } RObj;         /* rc: + 8-byte count */
typedef struct { GObj *p; uint64_t gen; } GRef;             /* genref: 16-byte reference */

static __declspec(noinline) void dangling(void) { fputs("generation mismatch\n", stderr); exit(3); }

static inline int64_t g_read(GRef r) {
    if (__builtin_expect(r.p->gen != r.gen, 0)) dangling();
    return r.p->a + r.p->b;
}
static inline int64_t f_read(GRef r) { return r.p->a + r.p->b; }   /* same layout, no check */
static inline RObj *r_take(RObj *p) { p->rc++; return p; }
static inline void r_drop(RObj *p) { if (--p->rc == 0) free(p); }

int main(int argc, char **argv) {
    const char *m = argc > 1 ? argv[1] : "";
    SetThreadAffinityMask(GetCurrentThread(), 2);   /* logical CPU 1, a P-core */
    h_start(); h_seed(42);
    int rnd = strstr(m, "rand") != NULL;
    uint64_t sum = 0;
    LARGE_INTEGER tw, te, fq;
    if (!strncmp(m, "raw", 3)) {
        Obj **refs = malloc(N * sizeof *refs);
        for (uint32_t i = 0; i < N; i++) { refs[i] = malloc(sizeof(Obj)); refs[i]->a = i; refs[i]->b = h_rand(); }
        QueryPerformanceCounter(&tw);
        if (rnd) for (uint32_t k = 0; k < HITS; k++) { Obj *o = refs[h_rand() & (N - 1)]; sum += o->a + o->b; }
        else for (int p = 0; p < PASSES; p++) for (uint32_t i = 0; i < N; i++) sum += refs[i]->a + refs[i]->b;
    } else if (!strncmp(m, "genref", 6) || !strncmp(m, "fat", 3)) {
        int check = m[0] == 'g';
        GRef *refs = malloc(N * sizeof *refs);
        for (uint32_t i = 0; i < N; i++) {
            GObj *o = malloc(sizeof(GObj)); o->gen = ((uint64_t)i * 0x9E3779B97F4A7C15ull) | 1;  /* random-looking, but leaves h_rand alone so every mode does the same work */ o->a = i; o->b = h_rand();
            refs[i].p = o; refs[i].gen = o->gen;
        }
        QueryPerformanceCounter(&tw);
        /* four separate loops, so the checked loops are the same code as without the fat mode */
        if (rnd && check) for (uint32_t k = 0; k < HITS; k++) sum += g_read(refs[h_rand() & (N - 1)]);
        else if (rnd) for (uint32_t k = 0; k < HITS; k++) sum += f_read(refs[h_rand() & (N - 1)]);
        else if (check) for (int p = 0; p < PASSES; p++) for (uint32_t i = 0; i < N; i++) sum += g_read(refs[i]);
        else for (int p = 0; p < PASSES; p++) for (uint32_t i = 0; i < N; i++) sum += f_read(refs[i]);
    } else if (!strncmp(m, "rc", 2)) {
        RObj **refs = malloc(N * sizeof *refs);
        for (uint32_t i = 0; i < N; i++) { refs[i] = malloc(sizeof(RObj)); refs[i]->rc = 1; refs[i]->a = i; refs[i]->b = h_rand(); }
        QueryPerformanceCounter(&tw);
        if (rnd) for (uint32_t k = 0; k < HITS; k++) { RObj *o = r_take(refs[h_rand() & (N - 1)]); sum += o->a + o->b; r_drop(o); }
        else for (int p = 0; p < PASSES; p++) for (uint32_t i = 0; i < N; i++) { RObj *o = r_take(refs[i]); sum += o->a + o->b; r_drop(o); }
    } else { fputs("mode: raw_seq raw_rand fat_seq fat_rand genref_seq genref_rand rc_seq rc_rand\n", stderr); return 2; }
    QueryPerformanceCounter(&te); QueryPerformanceFrequency(&fq);
    printf("%s work_ms=%.3f ", m, (double)(te.QuadPart - tw.QuadPart) * 1000.0 / (double)fq.QuadPart);
    h_report(m, sum);
    return 0;
}
