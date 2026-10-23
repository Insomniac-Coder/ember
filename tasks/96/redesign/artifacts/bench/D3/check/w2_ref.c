/* Independent reference for W2's checksum (parent and child counts as int arrays; not a model). */
#include "../../harness.h"
#define N 1000000
static int32_t PARENT[N], NCH[N];
int main(void) {
    h_start();
    h_seed(42);
    PARENT[0] = -1;
    for (int64_t i = 1; i < N; i++) { int64_t p = (int64_t)h_rand() % i; PARENT[i] = (int32_t)p; NCH[p]++; }
    uint64_t depth = 0;
    for (int64_t pass = 0; pass < 10; pass++) {
        for (int64_t i = 0; i < N; i++) for (int32_t d = PARENT[i]; d >= 0; d = PARENT[d]) depth++;
        for (int64_t i = 1; i < N; i++)
            if ((i + pass) % 100 == 0) {
                int64_t np = (int64_t)h_rand() % i;
                NCH[PARENT[i]]--; PARENT[i] = (int32_t)np; NCH[np]++;
            }
    }
    uint64_t sum = depth;
    for (int64_t i = 0; i < N; i++) sum += (uint64_t)((int64_t)(i % 1000) * NCH[i]);
    h_report("ref-w2", sum);
    return 0;
}
