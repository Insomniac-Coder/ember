/* `[BEN-6]` — the C function `ffi_direct_call.em` calls (`ffi_direct_call_mix.h`); its twin defines the same. */
#include "ffi_direct_call_mix.h"
#include <stdint.h>

int64_t ffi_mix(int64_t a, int64_t b) {
    return (int64_t)((uint64_t)a * 31u + (uint64_t)b);
}
