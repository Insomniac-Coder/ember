#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int64_t n = 1000000;
    int64_t* xs = (int64_t*)malloc(n * sizeof *xs);
    for (int64_t i = 0; i < n; i++) xs[i] = i % 1000;
    int64_t total = 0;
    for (int64_t round = 0; round < 300; round++) {
        int64_t taken = 0;
        for (int64_t i = round; i < n && taken < 300000; i += 3, taken++) total ^= xs[i];
    }
    printf("%lld\n", (long long)total);
    return 0;
}
