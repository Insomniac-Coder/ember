#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int64_t n = 100000;
    int64_t* restrict a = (int64_t*)malloc(n * sizeof *a);
    int64_t* restrict out = (int64_t*)malloc(n * sizeof *out);
    for (int64_t i = 0; i < n; i++) { a[i] = i % 1000; out[i] = 0; }
    for (int64_t round = 0; round < 2000; round++)
        for (int64_t i = 0; i < n; i++) out[i] = (out[i] + a[i]) & 1023;
    int64_t total = 0;
    for (int64_t i = 0; i < n; i++) total = total + out[i];
    printf("%lld\n", (long long)total);
    return 0;
}
