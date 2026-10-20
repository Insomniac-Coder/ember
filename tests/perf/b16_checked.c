#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int64_t n = 100000;
    /* Distinct page offsets avoid false load/store dependencies in MSVC's
       interchanged loop. Ordinary malloc placement made this same loop vary
       from about 0.5 to 7 seconds. The extra space covers alignment and offsets;
       the element count and loop work are identical (bench/README.md). */
    unsigned char* a_storage = malloc(n * sizeof(int64_t) + 8192);
    unsigned char* b_storage = malloc(n * sizeof(int64_t) + 8192);
    unsigned char* out_storage = malloc(n * sizeof(int64_t) + 8192);
    /* Derive pointers from their allocations, retaining the compiler's alias information. */
    int64_t* a = (int64_t*)(a_storage + ((-(uintptr_t)a_storage) & 4095));
    int64_t* b = (int64_t*)(b_storage + ((-(uintptr_t)b_storage) & 4095) + 1024);
    int64_t* out = (int64_t*)(out_storage + ((-(uintptr_t)out_storage) & 4095) + 2048);
    for (int64_t i = 0; i < n; i++) { a[i] = i % 1000; b[i] = i % 7; out[i] = 0; }
    for (int64_t round = 0; round < 20000; round++)
        for (int64_t i = 0; i < n; i++) out[i] = (((a[i] ^ round) + b[i]) - 7) & 1023;
    int64_t total = 0;
    for (int64_t i = 0; i < n; i++) total = total + out[i];
    printf("%lld\n", (long long)total);
    return 0;
}
