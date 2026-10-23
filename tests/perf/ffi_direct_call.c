#include <stdint.h>
#include <stdio.h>

#if defined(_MSC_VER)
#define NOINLINE __declspec(noinline)
#else
#define NOINLINE __attribute__((noinline))
#endif

/* As `ffi_direct_call_mix.c` defines it, called as the Ember program calls it: not inlined. */
NOINLINE int64_t ffi_mix(int64_t a, int64_t b) {
    return (int64_t)((uint64_t)a * 31u + (uint64_t)b);
}

int main(void) {
    uint64_t total = 0;
    for (int64_t i = 0; i < 200000000; i++) total += (uint64_t)ffi_mix(i, 3);
    printf("%lld\n", (long long)total);
    return 0;
}
