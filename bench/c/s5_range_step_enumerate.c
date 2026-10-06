#include <stdint.h>
#include <stdio.h>
int main(void) {
    int64_t n = 2000000;
    int64_t total = 0;
    for (int64_t round = 0; round < 300; round++) {
        int64_t i = 0;
        for (int64_t v = round; v < n; v += 2, i++) total ^= i ^ v;
    }
    printf("%lld\n", (long long)total);
    return 0;
}
