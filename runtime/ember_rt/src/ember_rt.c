/* ember_rt — implementation (Part XVIII §9). */

#include "ember_rt.h"

#include <inttypes.h>
#include <stdlib.h>
#include <string.h>

#if defined(_WIN32)
#include <malloc.h>
#endif

/* -- state ------------------------------------------------------------------
 *
 * [RT-2] No global constructors. The configuration is a plain static that
 * ember_rt_init fills in; before init it holds the defaults, so a runtime
 * function called from a static initialiser in host C++ code still behaves.
 */

static ember_rt_config g_config;
static bool g_initialised = false;
static ember_alloc_stats g_stats = { 0, 0, 0 };

ember_rt_config ember_rt_config_default(void) {
    ember_rt_config cfg;
    cfg.alloc = NULL;
    cfg.free = NULL;
    cfg.log = NULL;
    cfg.on_panic = NULL;
    cfg.flags = 0;
    return cfg;
}

int ember_rt_init(const ember_rt_config* cfg) {
    if (g_initialised) {
        return 0; /* idempotent */
    }
    g_config = cfg ? *cfg : ember_rt_config_default();
    g_initialised = true;
    ember_rt_thread_attach();
    return 0;
}

void ember_rt_shutdown(void) {
    if (!g_initialised) {
        return;
    }
    fflush(stdout);
    fflush(stderr);
    g_initialised = false;
}

uint32_t ember_rt_abi_version(void) { return EMBER_RT_ABI; }

/* Phase 0 is single-threaded. The entry points exist so that generated
 * trampolines ([FFI-22]) can call them unconditionally from the start. */
void ember_rt_thread_attach(void) {}
void ember_rt_thread_detach(void) {}

/* -- memory ------------------------------------------------------------------
 *
 * [HEAP-1] Everything that allocates goes through here, which is what makes
 * the `Alloc` effect ([EFF-*]) mean something: a `@noalloc` function is one
 * whose emitted C contains no call to these symbols.
 */

static void* raw_alloc(size_t size, size_t align) {
    if (size == 0) {
        size = 1;
    }
    if (align < sizeof(void*)) {
        align = sizeof(void*);
    }
#if defined(_MSC_VER)
    return _aligned_malloc(size, align);
#else
    /* C11 aligned_alloc requires a size that is a multiple of the alignment. */
    size_t rounded = (size + align - 1) & ~(align - 1);
    return aligned_alloc(align, rounded);
#endif
}

static void raw_free(void* p) {
    if (p == NULL) {
        return;
    }
#if defined(_MSC_VER)
    _aligned_free(p);
#else
    free(p);
#endif
}

void* ember_try_alloc(size_t size, size_t align) {
    void* p = g_config.alloc ? g_config.alloc(size, align) : raw_alloc(size, align);
    if (p != NULL) {
        g_stats.live_bytes += (uint64_t)size;
        g_stats.total_allocations += 1;
    }
    return p;
}

void* ember_alloc(size_t size, size_t align) {
    void* p = ember_try_alloc(size, align);
    if (p == NULL) {
        /* [ALC-4] Allocation failure is a panic for the standard containers;
         * try_reserve is the form that reports it as a value. */
        ember_loc loc = { "<allocator>", 0, 0 };
        ember_panic("out of memory", 13, loc);
    }
    return p;
}

void ember_free(void* p, size_t size, size_t align) {
    (void)align;
    if (p == NULL) {
        return;
    }
    if (g_config.free) {
        g_config.free(p, size, align);
    } else {
        raw_free(p);
    }
    if (g_stats.live_bytes >= (uint64_t)size) {
        g_stats.live_bytes -= (uint64_t)size;
    }
    g_stats.total_frees += 1;
}

void* ember_realloc(void* p, size_t old_size, size_t new_size, size_t align) {
    void* fresh = ember_alloc(new_size, align);
    if (p != NULL) {
        size_t copy = old_size < new_size ? old_size : new_size;
        memcpy(fresh, p, copy);
        ember_free(p, old_size, align);
    }
    return fresh;
}

void ember_debug_alloc_stats(ember_alloc_stats* out) {
    if (out) {
        *out = g_stats;
    }
}

/* -- panics ------------------------------------------------------------------ */

void ember_backtrace_print(void) {
    /* A real backtrace needs the platform unwinder; Phase 0 states the
     * limitation rather than printing something misleading. */
    fputs("note: backtraces are not available in this build\n", stderr);
}

static EMBER_NORETURN void panic_with(const char* msg, size_t len, ember_loc loc) {
    fflush(stdout);
    if (loc.file != NULL) {
        fprintf(stderr, "panic at %s:%u:%u: ", loc.file, loc.line, loc.column);
    } else {
        fputs("panic: ", stderr);
    }
    fwrite(msg, 1, len, stderr);
    fputc('\n', stderr);
    ember_backtrace_print();
    if (g_config.on_panic) {
        g_config.on_panic(msg, len);
    }
    fflush(stderr);
    abort();
}

void ember_panic(const char* msg, size_t len, ember_loc loc) { panic_with(msg, len, loc); }

void ember_panic_bounds(size_t index, size_t len, ember_loc loc) {
    char buffer[128];
    int written = snprintf(buffer, sizeof buffer,
                           "index %zu is out of bounds for a length of %zu", index, len);
    panic_with(buffer, written > 0 ? (size_t)written : 0, loc);
}

void ember_panic_overflow(const char* op, ember_loc loc) {
    char buffer[128];
    int written = snprintf(buffer, sizeof buffer, "integer overflow in `%s`", op);
    panic_with(buffer, written > 0 ? (size_t)written : 0, loc);
}

void ember_panic_div_zero(ember_loc loc) { panic_with("division by zero", 16, loc); }

void ember_panic_unwrap(const char* what, ember_loc loc) {
    char buffer[128];
    int written = snprintf(buffer, sizeof buffer, "unwrap on %s", what);
    panic_with(buffer, written > 0 ? (size_t)written : 0, loc);
}

/* -- printing -----------------------------------------------------------------
 *
 * Floats print as the shortest decimal that reads back as the same value, and
 * an integral value prints without a fractional part: 5.0f prints as "5".
 * Finding the shortest form by trying increasing precision is slower than Ryu
 * but is exact, is twenty lines rather than a thousand, and prints one number
 * per call. std.fmt replaces it in Phase 1.
 */

static void write_shortest_f64(double v, char* buffer, size_t cap) {
    for (int precision = 1; precision <= 17; ++precision) {
        snprintf(buffer, cap, "%.*g", precision, v);
        if (strtod(buffer, NULL) == v) {
            return;
        }
    }
    snprintf(buffer, cap, "%.17g", v);
}

static void write_shortest_f32(float v, char* buffer, size_t cap) {
    for (int precision = 1; precision <= 9; ++precision) {
        snprintf(buffer, cap, "%.*g", precision, (double)v);
        if (strtof(buffer, NULL) == v) {
            return;
        }
    }
    snprintf(buffer, cap, "%.9g", (double)v);
}

/* Encode one Unicode scalar value as UTF-8. Returns the byte count. */
static size_t encode_utf8(uint32_t c, unsigned char* out) {
    if (c < 0x80) {
        out[0] = (unsigned char)c;
        return 1;
    }
    if (c < 0x800) {
        out[0] = (unsigned char)(0xC0 | (c >> 6));
        out[1] = (unsigned char)(0x80 | (c & 0x3F));
        return 2;
    }
    if (c < 0x10000) {
        out[0] = (unsigned char)(0xE0 | (c >> 12));
        out[1] = (unsigned char)(0x80 | ((c >> 6) & 0x3F));
        out[2] = (unsigned char)(0x80 | (c & 0x3F));
        return 3;
    }
    out[0] = (unsigned char)(0xF0 | (c >> 18));
    out[1] = (unsigned char)(0x80 | ((c >> 12) & 0x3F));
    out[2] = (unsigned char)(0x80 | ((c >> 6) & 0x3F));
    out[3] = (unsigned char)(0x80 | (c & 0x3F));
    return 4;
}

void ember_print_str(ember_str s) {
    if (s.len > 0) {
        fwrite(s.ptr, 1, s.len, stdout);
    }
}

void ember_println_str(ember_str s) {
    ember_print_str(s);
    fputc('\n', stdout);
}

void ember_print_i64(int64_t v) { printf("%" PRId64, v); }
void ember_println_i64(int64_t v) { printf("%" PRId64 "\n", v); }
void ember_print_u64(uint64_t v) { printf("%" PRIu64, v); }
void ember_println_u64(uint64_t v) { printf("%" PRIu64 "\n", v); }

void ember_print_f32(float v) {
    char buffer[32];
    write_shortest_f32(v, buffer, sizeof buffer);
    fputs(buffer, stdout);
}

void ember_println_f32(float v) {
    ember_print_f32(v);
    fputc('\n', stdout);
}

void ember_print_f64(double v) {
    char buffer[32];
    write_shortest_f64(v, buffer, sizeof buffer);
    fputs(buffer, stdout);
}

void ember_println_f64(double v) {
    ember_print_f64(v);
    fputc('\n', stdout);
}

void ember_print_bool(bool v) { fputs(v ? "true" : "false", stdout); }

void ember_println_bool(bool v) {
    ember_print_bool(v);
    fputc('\n', stdout);
}

void ember_print_char(uint32_t v) {
    unsigned char buffer[4];
    size_t n = encode_utf8(v, buffer);
    fwrite(buffer, 1, n, stdout);
}

void ember_println_char(uint32_t v) {
    ember_print_char(v);
    fputc('\n', stdout);
}
