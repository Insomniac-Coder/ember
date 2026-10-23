/* phase 2 (Hylo / C#): what two "smarter solutions" cost against plain C.
   1. Paged storage (pages never move, so a reference into a pool survives an append):
      append, sequential sum and random access, contiguous array vs 4096-element pages.
   2. Index-as-reference (Racordon 2025; Hylo/MVS has no stored references):
      random deref through a raw pointer vs through a generation-checked handle.
   One mode per process so peak_kb belongs to that mode; work_ms times the phase after the fill
   (for *_append, ms is the fill itself).
   Build: clang -O2 paged_vs_handles.c -o paged_vs_handles.exe   Run: paged_vs_handles.exe <mode> */
#include "../bench/harness.h"
#include <string.h>

typedef struct { int64_t a, b; } Comp;   /* a 16-byte component */
#define N      (1u << 22)                /* 4,194,304 components = 64 MiB */
#ifndef PSHIFT
#define PSHIFT 12                        /* 4096-element pages; -DPSHIFT=16 for 65536 */
#endif
#define PAGE   (1u << PSHIFT)
#define PASSES 20
#define HITS   (1u << 24)                /* 16,777,216 random accesses */

/* contiguous pool: grows by doubling with realloc (moves; a reference into it would dangle) */
static Comp *dense; static uint32_t dlen, dcap;
static void dpush(Comp c) {
    if (dlen == dcap) { dcap = dcap ? dcap * 2 : 16; dense = realloc(dense, (size_t)dcap * sizeof(Comp)); }
    dense[dlen++] = c;
}
/* paged pool: a table of fixed pages; a page never moves once allocated */
static Comp **pages; static uint32_t plen, ptab;
static void ppush(Comp c) {
    if ((plen & (PAGE - 1)) == 0) {
        uint32_t p = plen >> PSHIFT;
        if (p == ptab) { ptab = ptab ? ptab * 2 : 16; pages = realloc(pages, (size_t)ptab * sizeof(Comp *)); }
        pages[p] = malloc(PAGE * sizeof(Comp));
    }
    pages[plen >> PSHIFT][plen & (PAGE - 1)] = c;
    plen++;
}

int main(int argc, char **argv) {
    const char *m = argc > 1 ? argv[1] : "";
    h_start(); h_seed(42);
    uint64_t sum = 0;
    int paged = strstr(m, "paged") != NULL;
    LARGE_INTEGER tw, te, fq;
    for (uint32_t i = 0; i < N; i++) {
        Comp c = { i, (int64_t)h_rand() };
        if (paged) ppush(c); else dpush(c);
    }
    QueryPerformanceCounter(&tw);                        /* work phase starts after the fill */
    if (!strcmp(m, "contig_append") || !strcmp(m, "paged_append")) {
        sum = paged ? (uint64_t)pages[(N - 1) >> PSHIFT][(N - 1) & (PAGE - 1)].b : (uint64_t)dense[N - 1].b;
    } else if (!strcmp(m, "contig_iter")) {
        for (int p = 0; p < PASSES; p++) for (uint32_t i = 0; i < dlen; i++) sum += dense[i].a + dense[i].b;
    } else if (!strcmp(m, "paged_iter")) {           /* page by page, as EnTT's view iterates */
        for (int p = 0; p < PASSES; p++)
            for (uint32_t pg = 0; pg * PAGE < plen; pg++) {
                Comp *q = pages[pg]; uint32_t n = plen - pg * PAGE < PAGE ? plen - pg * PAGE : PAGE;
                for (uint32_t i = 0; i < n; i++) sum += q[i].a + q[i].b;
            }
    } else if (!strcmp(m, "contig_rand")) {
        for (uint32_t k = 0; k < HITS; k++) { uint32_t i = h_rand() & (N - 1); sum += dense[i].b; }
    } else if (!strcmp(m, "paged_rand")) {
        for (uint32_t k = 0; k < HITS; k++) { uint32_t i = h_rand() & (N - 1); sum += pages[i >> PSHIFT][i & (PAGE - 1)].b; }
    } else if (!strcmp(m, "ptr_deref") || !strcmp(m, "handle_deref")) {
        /* refs: 2^20 stored references to random components, then HITS derefs through them */
        enum { R = 1u << 20 };
        int handles = !strcmp(m, "handle_deref");
        uint32_t *gen = NULL;                             /* per-slot generation, as a handle pool keeps */
        if (handles) { gen = malloc(N * sizeof(uint32_t)); for (uint32_t i = 0; i < N; i++) gen[i] = 1; }
        Comp **ptrs = handles ? NULL : malloc(R * sizeof(Comp *));
        uint64_t *hs = handles ? malloc(R * sizeof(uint64_t)) : NULL;   /* u32 index | u32 generation */
        for (uint32_t r = 0; r < R; r++) {
            uint32_t i = h_rand() & (N - 1);
            if (handles) hs[r] = i | ((uint64_t)gen[i] << 32); else ptrs[r] = &dense[i];
        }
        QueryPerformanceCounter(&tw);                    /* time only the derefs */
        for (uint32_t k = 0; k < HITS; k++) {
            uint32_t r = h_rand() & (R - 1);
            if (handles) {
                uint64_t h = hs[r]; uint32_t i = (uint32_t)h;
                if (gen[i] != (uint32_t)(h >> 32)) { fputs("stale handle\n", stderr); abort(); }  /* the panic */
                sum += dense[i].b;
            } else sum += ptrs[r]->b;
        }
    } else if (!strcmp(m, "handle_inline_deref")) {
        /* the generation stored beside the value (same cache line): 24-byte slots instead of 16 */
        enum { R = 1u << 20 };
        typedef struct { int64_t a, b; uint32_t gen; } GComp;
        GComp *slots = malloc((size_t)N * sizeof(GComp));
        for (uint32_t i = 0; i < N; i++) { slots[i].a = dense[i].a; slots[i].b = dense[i].b; slots[i].gen = 1; }
        free(dense); dense = NULL;
        uint64_t *hs = malloc(R * sizeof(uint64_t));
        for (uint32_t r = 0; r < R; r++) { uint32_t i = h_rand() & (N - 1); hs[r] = i | ((uint64_t)slots[i].gen << 32); }
        QueryPerformanceCounter(&tw);
        for (uint32_t k = 0; k < HITS; k++) {
            uint64_t h = hs[h_rand() & (R - 1)]; GComp *s = &slots[(uint32_t)h];
            if (s->gen != (uint32_t)(h >> 32)) { fputs("stale handle\n", stderr); abort(); }
            sum += s->b;
        }
    } else { fputs("modes: contig_append paged_append contig_iter paged_iter contig_rand paged_rand ptr_deref handle_deref handle_inline_deref\n", stderr); return 2; }
    QueryPerformanceCounter(&te); QueryPerformanceFrequency(&fq);
    printf("%s work_ms=%.3f ", m, (double)(te.QuadPart - tw.QuadPart) * 1000.0 / (double)fq.QuadPart);
    h_report(m, sum);
    return 0;
}
