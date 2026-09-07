/* ember_rt — the Ember runtime, C11 (Part XVIII §9).
 *
 * [RT-1] No dependencies beyond libc and the OS.
 * [RT-2] No global constructors; ember_rt_init is explicit and idempotent.
 *
 * Phase 0 provides allocation, panics and printing. Objects, reference
 * counting, exclusivity, threads and arenas arrive with the phases that give
 * them meaning. The declarations that Part XVIII §9 fixes are written here as
 * they will be, so that the ABI does not move under callers later.
 */

#ifndef EMBER_RT_H
#define EMBER_RT_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#ifdef __cplusplus
extern "C" {
#endif

/* The ABI version macro of Part XVIII §9. Bumped when anything below changes
 * shape; ember_rt_abi_version() reports what the linked library was built as. */
#define EMBER_RT_ABI 1

/* -- portability shims ---------------------------------------------------- */

#if defined(_MSC_VER)
#define EMBER_UNREACHABLE() __assume(0)
#define EMBER_NORETURN __declspec(noreturn)
#define EMBER_INLINE __forceinline
#else
#define EMBER_UNREACHABLE() __builtin_unreachable()
#define EMBER_NORETURN __attribute__((noreturn))
#define EMBER_INLINE __attribute__((always_inline)) inline
#endif

#define EMBER_INF_F64 (1.0 / 0.0)
#define EMBER_NAN_F32 (0.0f / 0.0f)
#define EMBER_NAN_F64 (0.0 / 0.0)

/* -- str ------------------------------------------------------------------ */

/* [LEX-20] A string literal has type `str`: a view over UTF-8 bytes with the
 * static region. Pointer plus length, never NUL-terminated by construction. */
typedef struct ember_str {
    const unsigned char* ptr;
    size_t len;
} ember_str;

#define ember_str_lit(text, length) \
    ((ember_str){ (const unsigned char*)(text), (size_t)(length) })

/* -- source locations ------------------------------------------------------ */

typedef struct ember_loc {
    const char* file;
    uint32_t line;
    uint32_t column;
} ember_loc;

/* -- memory ---------------------------------------------------------------- */

/* [HEAP-1] Every heap type allocates through these. ember_alloc never returns
 * NULL: allocation failure is a panic ([ALC-4]). */
void* ember_alloc(size_t size, size_t align);
void* ember_realloc(void* p, size_t old_size, size_t new_size, size_t align);
void ember_free(void* p, size_t size, size_t align);
void* ember_try_alloc(size_t size, size_t align);

/* -- panics ---------------------------------------------------------------- */

/* [RT-4] A panic prints "panic at <file>:<line>:<col>: <message>", then a
 * backtrace in debug and release, then calls cfg.on_panic and abort(). */
EMBER_NORETURN void ember_panic(const char* msg, size_t len, ember_loc loc);
EMBER_NORETURN void ember_panic_bounds(size_t index, size_t len, ember_loc loc);
EMBER_NORETURN void ember_panic_overflow(const char* op, ember_loc loc);
EMBER_NORETURN void ember_panic_div_zero(ember_loc loc);
EMBER_NORETURN void ember_panic_unwrap(const char* what, ember_loc loc);
void ember_backtrace_print(void);

/* -- printing -------------------------------------------------------------- */

/* Phase 0 has no `Display` interface, so the compiler picks a printer from the
 * argument's type. These are replaced by std.fmt in Phase 1; they stay for the
 * runtime's own diagnostics.
 *
 * [STD-2] println of a literal or a str does not allocate. */
void ember_print_str(ember_str s);
void ember_println_str(ember_str s);
void ember_print_i64(int64_t v);
void ember_println_i64(int64_t v);
void ember_print_u64(uint64_t v);
void ember_println_u64(uint64_t v);
void ember_print_f32(float v);
void ember_println_f32(float v);
void ember_print_f64(double v);
void ember_println_f64(double v);
void ember_print_bool(bool v);
void ember_println_bool(bool v);
void ember_print_char(uint32_t v);
void ember_println_char(uint32_t v);

/* -- lifecycle and embedding ------------------------------------------------ */

/* [FFI-27] The embedding API. A host may route allocation and logging into its
 * own systems; RageV's plan is cfg.log = its logger with the allocator left at
 * the default. */
typedef struct ember_rt_config {
    void* (*alloc)(size_t size, size_t align);
    void (*free)(void* p, size_t size, size_t align);
    void (*log)(int level, const char* msg, size_t len);
    void (*on_panic)(const char* msg, size_t len);
    uint32_t flags;
} ember_rt_config;

ember_rt_config ember_rt_config_default(void);
int ember_rt_init(const ember_rt_config* cfg);
void ember_rt_shutdown(void);
uint32_t ember_rt_abi_version(void);

/* Per-thread attach. Idempotent and cheap after the first call ([FFI-22]).
 * Phase 0 is single-threaded; the entry points exist so that generated
 * trampolines can call them from the start. */
void ember_rt_thread_attach(void);
void ember_rt_thread_detach(void);

/* -- debug facilities ------------------------------------------------------- */

typedef struct ember_alloc_stats {
    uint64_t live_bytes;
    uint64_t total_allocations;
    uint64_t total_frees;
} ember_alloc_stats;

void ember_debug_alloc_stats(ember_alloc_stats* out);

#ifdef __cplusplus
}
#endif

#endif /* EMBER_RT_H */
