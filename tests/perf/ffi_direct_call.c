#include <stdint.h>
#include <stdio.h>

/* `[BEN-6]` — `ffi_mix` is kept as a function compiled apart is: callable as written by code the
 * compiler cannot see. The Ember program calls the one `ffi_direct_call_mix.c` compiles apart
 * (D-897), whose body no optimiser of the program sees; with `noinline` alone, the twin's
 * whole-program build (`/GL`, `-flto`) still sees this one, and rewrites the call: it folds the
 * constant `3` into a copy of the function (clang, GCC's `ffi_mix.constprop.0`, MSVC), and keeps the
 * running total in a register that copy leaves alone (GCC, MSVC). Each compiler's own spelling stops
 * that: an exported function (MSVC, clang-cl), a `used` one (clang), `noipa` (GCC). */
#if defined(_MSC_VER)
#define KEPT __declspec(noinline) __declspec(dllexport)
#elif defined(__clang__)
#define KEPT __attribute__((noinline, used))
#else
#define KEPT __attribute__((noipa))
#endif

/* As `ffi_direct_call_mix.c` defines it, called as the Ember program calls it. */
KEPT int64_t ffi_mix(int64_t a, int64_t b) {
    return (int64_t)((uint64_t)a * 31u + (uint64_t)b);
}

int main(void) {
    uint64_t total = 0;
    for (int64_t i = 0; i < 200000000; i++) total += (uint64_t)ffi_mix(i, 3);
    printf("%lld\n", (long long)total);
    return 0;
}
