/* Independent reference for W3's checksum: arrays indexed by id, target as an id, ids never reused,
   so "target destroyed" is ALIVE[target] == 0. Not a model, not timed. */
#include "../../harness.h"
#include <stdlib.h>
#define M 100000
#define CAP 20000000
static int32_t X[CAP], Y[CAP], H[CAP], KIND[CAP];
static int64_t TGT[CAP];
static unsigned char ALIVE[CAP];
static int64_t LIVE[CAP]; /* live ids in increasing order */
static int64_t DEAD[CAP];
static int sgn(int32_t d) { return (d > 0) - (d < 0); }
int main(void) {
    h_start();
    h_seed(7);
    int64_t nlive = 0;
    for (int64_t i = 0; i < M; i++) {
        KIND[i] = (int32_t)(i % 3); X[i] = (int32_t)(h_rand() % 1000); Y[i] = (int32_t)(h_rand() % 1000);
        H[i] = (int32_t)(100 + i % 50); TGT[i] = i == 0 ? -1 : (int64_t)(h_rand() % (uint64_t)i);
        ALIVE[i] = 1; LIVE[nlive++] = i;
    }
    int64_t next = M, total = 0;
    for (int f = 0; f < 1000; f++) {
        for (int64_t k = 0; k < nlive; k++) {
            int64_t i = LIVE[k]; int64_t t = TGT[i];
            int has = t >= 0 && ALIVE[t];
            if (KIND[i] == 0) { if (has) { X[i] += sgn(X[t] - X[i]); Y[i] += sgn(Y[t] - Y[i]); } }
            else if (KIND[i] == 1) X[i] = (X[i] + 1) % 1000;
            else if (has && abs(X[t] - X[i]) + abs(Y[t] - Y[i]) < 300) H[t] -= 1;
        }
        int64_t nd = 0, j = 0;
        for (int64_t k = 0; k < nlive; k++) {
            int64_t i = LIVE[k];
            if (H[i] <= 0) { ALIVE[i] = 0; DEAD[nd++] = i; } else LIVE[j++] = i;
        }
        nlive = j; total += nd;
        for (int64_t d = 0; d < nd; d++) {
            int64_t id = next++;
            if (id >= CAP) { puts("CAP"); return 1; }
            KIND[id] = (int32_t)(id % 3); X[id] = (int32_t)(h_rand() % 1000); Y[id] = (int32_t)(h_rand() % 1000);
            H[id] = (int32_t)(100 + id % 50);
            TGT[id] = nlive > 0 ? LIVE[(int64_t)h_rand() % nlive] : -1;
            ALIVE[id] = 1; LIVE[nlive++] = id;
        }
    }
    int64_t sum = 0;
    for (int64_t k = 0; k < nlive; k++) { int64_t i = LIVE[k]; sum += (int64_t)X[i] + Y[i] + H[i]; }
    h_report("ref-w3", (uint64_t)sum + 1000000ull * (uint64_t)total);
    return 0;
}
