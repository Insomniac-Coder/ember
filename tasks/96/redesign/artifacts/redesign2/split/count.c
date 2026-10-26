/* Plain C baseline, W1 (ECS). No safety checks. Entities with a Velocity live in one dense
   array of movers (id, position, velocity); entities with only a Position live in a second
   array that never changes. A despawn is a swap-remove; the loop runs backwards so the element
   swapped in has already been visited, and a spawn is appended at the end, past the loop. */
#include "harness.h"
#define N 1000000
typedef struct { uint32_t id; float x, y, dx, dy; } Mover;
typedef struct { float x, y; } Pos;

int main(void) {
    h_start();
    Pos *still = malloc(sizeof(Pos) * (N / 2));
    size_t nstill = 0, nmov = 0, cap = N / 2 + 1;
    Mover *mov = malloc(sizeof(Mover) * cap);
    for (uint32_t i = 0; i < N; i++) {
        float x = (float)(i % 1000), y = (float)(i / 1000);
        if (i % 2 == 0) mov[nmov++] = (Mover){i, x, y, (float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f};
        else still[nstill++] = (Pos){x, y};
    }
    uint32_t next = N;
    { LARGE_INTEGER t, f; QueryPerformanceCounter(&t); QueryPerformanceFrequency(&f); printf("setup_ms=%.3f ", (double)(t.QuadPart - h_t0.QuadPart) * 1000.0 / (double)f.QuadPart); }
    for (uint32_t s = 0; s < 100; s++) {
        for (size_t j = nmov; j-- > 0;) {
            Mover *m = &mov[j];
            m->x += m->dx; m->y += m->dy;
            if (m->id % 100 == s) {
                *m = mov[--nmov];                          /* despawn */
                if (nmov == cap) { cap *= 2; mov = realloc(mov, sizeof(Mover) * cap); }
                mov[nmov++] = (Mover){next++, 0, 0, 1, 1}; /* spawn */
            }
        }
    }
    uint64_t sum = 0;
    for (size_t j = 0; j < nmov; j++) sum += (uint64_t)((int64_t)(mov[j].x * 4.0f) + (int64_t)(mov[j].y * 4.0f));
    for (size_t j = 0; j < nstill; j++) sum += (uint64_t)((int64_t)(still[j].x * 4.0f) + (int64_t)(still[j].y * 4.0f));
    h_report("plain-w1", sum + nmov + nstill); printf("spawns=%u\n", next - N);
    return 0;
}
