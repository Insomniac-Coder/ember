/* Independent reference for W1's checksum (arrays indexed by id; not a model, not timed). */
#include "../../harness.h"
#define N 1000000
#define CAP 4000000
static float X[CAP], Y[CAP], DX[CAP], DY[CAP];
static unsigned char ALIVE[CAP], VEL[CAP];
int main(void) {
    h_start();
    int64_t next = 0;
    for (int64_t i = 0; i < N; i++) {
        X[i] = (float)(i % 1000); Y[i] = (float)(i / 1000); ALIVE[i] = 1;
        if (i % 2 == 0) { VEL[i] = 1; DX[i] = (float)(i % 7) * 0.5f; DY[i] = (float)(i % 5) * 0.25f; }
        next++;
    }
    for (int64_t s = 0; s < 100; s++) {
        int64_t n0 = next;
        for (int64_t i = 0; i < n0; i++) {
            if (!ALIVE[i] || !VEL[i]) continue;
            X[i] += DX[i]; Y[i] += DY[i];
            if (i % 100 == s) {
                ALIVE[i] = 0;
                if (next >= CAP) { puts("CAP"); return 1; }
                X[next] = 0; Y[next] = 0; DX[next] = 1; DY[next] = 1; ALIVE[next] = 1; VEL[next] = 1; next++;
            }
        }
    }
    uint64_t sum = 0, live = 0;
    for (int64_t i = 0; i < next; i++)
        if (ALIVE[i]) { sum += (uint64_t)((int64_t)(X[i] * 4.0f) + (int64_t)(Y[i] * 4.0f)); live++; }
    h_report("ref-w1", sum + live);
    return 0;
}
