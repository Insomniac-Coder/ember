#include <stdint.h>
#include <stdio.h>
int main(void) {
    int64_t total = 0;
    for (int64_t i = 1; i < 50000001; i++) total += (i * 7) % 13 + i / 3 - i % 5;
    printf("%lld\n", (long long)total);
    return 0;
}
