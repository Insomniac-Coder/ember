/* A3 workload: two elements of one list changed by one call, collide(bodies[i], bodies[j]).
   4096 bodies (64 KB, fits in L2, so the arithmetic and the loads show, not cache misses),
   50,000,000 collisions between random distinct pairs (j != i always, so every variant does the same
   work and prints the same checksum). collide swaps the velocities and pushes the two apart.
   Variants differ only in what the compiler is told about the two pointers (COLLIDE_QUAL) and
   whether i == j is checked first (CHECK_DISTINCT). */
#include "../harness.h"
#define NB 4096
#define PAIRS 50000000
typedef struct { float x, y, vx, vy; } Body;
static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#ifndef NOINLINE
#define INL static inline
#else
#define INL static __attribute__((noinline))
#endif
INL void collide(Body *COLLIDE_QUAL a, Body *COLLIDE_QUAL b) {
    float tvx = a->vx, tvy = a->vy;
    a->vx = b->vx; a->vy = b->vy;
    b->vx = tvx;   b->vy = tvy;
    float dx = b->x - a->x, dy = b->y - a->y;
    a->x -= dx * 0.01f; a->y -= dy * 0.01f;
    b->x += dx * 0.01f; b->y += dy * 0.01f;
}
static int run(const char *name) {
    h_start();
    static Body bodies[NB];
    h_seed(7);
    for (int k = 0; k < NB; k++)
        bodies[k] = (Body){(float)(k % 64), (float)(k / 64), (float)(h_rand() % 7) - 3.0f, (float)(h_rand() % 5) - 2.0f};
    for (int n = 0; n < PAIRS; n++) {
        uint32_t i = h_rand() % NB, j = (i + 1 + h_rand() % (NB - 1)) % NB;
#ifdef CHECK_DISTINCT
        if (__builtin_expect(i == j, 0)) panic("the same body passed twice to be changed");
#endif
        collide(&bodies[i], &bodies[j]);
    }
    uint64_t s = 0;
    for (int k = 0; k < NB; k++) s += (uint64_t)((int64_t)(bodies[k].x * 16.0f) + (int64_t)(bodies[k].vx * 16.0f));
    h_report(name, s);
    return 0;
}
