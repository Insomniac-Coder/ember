/* Independent reference for W4's checksum: parallel arrays, no closures. Not a model, not timed. */
#include "../../harness.h"
#include <string.h>
#define H 10000
static int64_t CNT[H], MUL[H];
int main(void) {
    h_start();
    int64_t n = H, total = 0;
    for (int64_t i = 0; i < H; i++) { CNT[i] = 0; MUL[i] = i % 13 + 1; }
    for (int64_t k = 0; k < 10000000; k++) {
        int64_t i = k % n, v = k % 1000;
        CNT[i] += v * MUL[i];
        total += 1;
        if (k % 100000 == 99999) {
            memmove(&CNT[i], &CNT[i + 1], (size_t)(n - i - 1) * 8);
            memmove(&MUL[i], &MUL[i + 1], (size_t)(n - i - 1) * 8);
            CNT[n - 1] = 0; MUL[n - 1] = k % 17 + 1;
        }
    }
    int64_t s = 0;
    for (int64_t i = 0; i < n; i++) s += CNT[i];
    h_report("ref-w4", (uint64_t)(total + s));
    return 0;
}
