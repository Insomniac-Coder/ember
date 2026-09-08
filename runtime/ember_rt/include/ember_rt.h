/* ember_rt — the Ember runtime, C11 (Part XVIII §9).
 *
 * [RT-1] No dependencies beyond libc and the OS.
 * [RT-2] No global constructors; ember_rt_init is explicit and idempotent.
 *
 * Objects, reference counting, exclusivity, threads and arenas arrive with the
 * phases that give them meaning. The declarations Part XVIII §9 fixes are
 * written here as they will be, so the ABI does not move under callers later.
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

static inline ember_loc ember_loc_at(const char* file, uint32_t line, uint32_t column) {
    ember_loc loc;
    loc.file = file;
    loc.line = line;
    loc.column = column;
    return loc;
}

static inline ember_loc ember_loc_unknown(void) { return ember_loc_at(NULL, 0, 0); }

/* -- checked arithmetic ------------------------------------------------------
 *
 * [TYP-8] Under the `panic` overflow policy the compiler emits one of these per
 * arithmetic operation and then asserts on the result. Each writes the
 * two's-complement (wrapped) result whatever happens — so the destination is
 * defined on both paths — and returns whether the true result was out of range.
 *
 * Clang and GCC have __builtin_*_overflow, which lowers to a single flag test.
 * MSVC has no equivalent, so the fallbacks compute in a wider type and
 * range-check. Both are exact; only the code size differs.
 *
 * Signed arithmetic goes through the unsigned type of the same width, because
 * conversion is modular and signed overflow is undefined behaviour in C.
 */

#define EMBER_SMAX(BITS) ((int64_t)((((uint64_t)1) << ((BITS) - 1)) - 1))
#define EMBER_SMIN(BITS) (-EMBER_SMAX(BITS) - 1)
#define EMBER_UMAX(BITS) ((BITS) == 64 ? UINT64_MAX : ((((uint64_t)1) << (BITS)) - 1))

#if defined(__GNUC__) || defined(__clang__)

#define EMBER_CHECKED_OPS(SUFFIX, TYPE)                                      \
    static inline bool ember_ck_add_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        return __builtin_add_overflow(a, b, out);                            \
    }                                                                        \
    static inline bool ember_ck_sub_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        return __builtin_sub_overflow(a, b, out);                            \
    }                                                                        \
    static inline bool ember_ck_mul_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        return __builtin_mul_overflow(a, b, out);                            \
    }

#else

/* Narrow signed types: compute in int64_t, which cannot itself overflow for
 * operands of 32 bits or fewer, then range-check. */
#define EMBER_CHECKED_OPS_SNARROW(SUFFIX, TYPE, BITS)                        \
    static inline bool ember_ck_add_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        int64_t wide = (int64_t)a + (int64_t)b;                              \
        *out = (TYPE)wide;                                                   \
        return wide < EMBER_SMIN(BITS) || wide > EMBER_SMAX(BITS);           \
    }                                                                        \
    static inline bool ember_ck_sub_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        int64_t wide = (int64_t)a - (int64_t)b;                              \
        *out = (TYPE)wide;                                                   \
        return wide < EMBER_SMIN(BITS) || wide > EMBER_SMAX(BITS);           \
    }                                                                        \
    static inline bool ember_ck_mul_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        int64_t wide = (int64_t)a * (int64_t)b;                              \
        *out = (TYPE)wide;                                                   \
        return wide < EMBER_SMIN(BITS) || wide > EMBER_SMAX(BITS);           \
    }

#define EMBER_CHECKED_OPS_UNARROW(SUFFIX, TYPE, BITS)                        \
    static inline bool ember_ck_add_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        uint64_t wide = (uint64_t)a + (uint64_t)b;                           \
        *out = (TYPE)wide;                                                   \
        return wide > EMBER_UMAX(BITS);                                      \
    }                                                                        \
    static inline bool ember_ck_sub_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        *out = (TYPE)((uint64_t)a - (uint64_t)b);                            \
        return a < b;                                                        \
    }                                                                        \
    static inline bool ember_ck_mul_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        uint64_t wide = (uint64_t)a * (uint64_t)b;                           \
        *out = (TYPE)wide;                                                   \
        return wide > EMBER_UMAX(BITS);                                      \
    }

/* 64-bit: nothing wider to compute in, so check the operands directly. */
#define EMBER_CHECKED_OPS_S64(SUFFIX, TYPE)                                  \
    static inline bool ember_ck_add_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        *out = (TYPE)((uint64_t)a + (uint64_t)b);                            \
        return (b > 0 && a > (TYPE)(INT64_MAX - b))                          \
            || (b < 0 && a < (TYPE)(INT64_MIN - b));                         \
    }                                                                        \
    static inline bool ember_ck_sub_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        *out = (TYPE)((uint64_t)a - (uint64_t)b);                            \
        return (b < 0 && a > (TYPE)(INT64_MAX + b))                          \
            || (b > 0 && a < (TYPE)(INT64_MIN + b));                         \
    }                                                                        \
    static inline bool ember_ck_mul_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        *out = (TYPE)((uint64_t)a * (uint64_t)b);                            \
        if (a == 0 || b == 0) { return false; }                              \
        if (a == (TYPE)-1) { return b == (TYPE)INT64_MIN; }                  \
        if (b == (TYPE)-1) { return a == (TYPE)INT64_MIN; }                  \
        return (TYPE)(*out) / b != a;                                        \
    }

#define EMBER_CHECKED_OPS_U64(SUFFIX, TYPE)                                  \
    static inline bool ember_ck_add_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        *out = (TYPE)(a + b);                                                \
        return *out < a;                                                     \
    }                                                                        \
    static inline bool ember_ck_sub_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        *out = (TYPE)(a - b);                                                \
        return a < b;                                                        \
    }                                                                        \
    static inline bool ember_ck_mul_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        *out = (TYPE)(a * b);                                                \
        if (a == 0) { return false; }                                        \
        return *out / a != b;                                                \
    }

#endif /* builtin overflow */

#if defined(__GNUC__) || defined(__clang__)
EMBER_CHECKED_OPS(i8, int8_t)
EMBER_CHECKED_OPS(i16, int16_t)
EMBER_CHECKED_OPS(i32, int32_t)
EMBER_CHECKED_OPS(i64, int64_t)
EMBER_CHECKED_OPS(isize, ptrdiff_t)
EMBER_CHECKED_OPS(u8, uint8_t)
EMBER_CHECKED_OPS(u16, uint16_t)
EMBER_CHECKED_OPS(u32, uint32_t)
EMBER_CHECKED_OPS(u64, uint64_t)
EMBER_CHECKED_OPS(usize, size_t)
#else
EMBER_CHECKED_OPS_SNARROW(i8, int8_t, 8)
EMBER_CHECKED_OPS_SNARROW(i16, int16_t, 16)
EMBER_CHECKED_OPS_SNARROW(i32, int32_t, 32)
EMBER_CHECKED_OPS_S64(i64, int64_t)
EMBER_CHECKED_OPS_S64(isize, ptrdiff_t)
EMBER_CHECKED_OPS_UNARROW(u8, uint8_t, 8)
EMBER_CHECKED_OPS_UNARROW(u16, uint16_t, 16)
EMBER_CHECKED_OPS_UNARROW(u32, uint32_t, 32)
EMBER_CHECKED_OPS_U64(u64, uint64_t)
EMBER_CHECKED_OPS_U64(usize, size_t)
#endif

/* Division. The compiler has already emitted the zero-divisor check, so the
 * only thing left to report is T.MIN / -1, whose true result is not
 * representable ([TYP-8]). Unsigned division cannot overflow and never gets
 * here. */
#define EMBER_CHECKED_DIV(SUFFIX, TYPE, MINVAL)                              \
    static inline bool ember_ck_div_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        if (a == (TYPE)(MINVAL) && b == (TYPE)-1) { *out = a; return true; } \
        *out = (TYPE)(a / b);                                                \
        return false;                                                        \
    }                                                                        \
    static inline bool ember_ck_rem_##SUFFIX(TYPE a, TYPE b, TYPE* out) {    \
        if (a == (TYPE)(MINVAL) && b == (TYPE)-1) { *out = 0; return true; } \
        *out = (TYPE)(a % b);                                                \
        return false;                                                        \
    }

EMBER_CHECKED_DIV(i8, int8_t, INT8_MIN)
EMBER_CHECKED_DIV(i16, int16_t, INT16_MIN)
EMBER_CHECKED_DIV(i32, int32_t, INT32_MIN)
EMBER_CHECKED_DIV(i64, int64_t, INT64_MIN)
EMBER_CHECKED_DIV(isize, ptrdiff_t, INT64_MIN)

/* -- memory ---------------------------------------------------------------- */

/* [HEAP-1] Every heap type allocates through these. ember_alloc never returns
 * NULL: allocation failure is a panic ([ALC-4]). */
void* ember_alloc(size_t size, size_t align);
void* ember_realloc(void* p, size_t old_size, size_t new_size, size_t align);
void ember_free(void* p, size_t size, size_t align);
void* ember_try_alloc(size_t size, size_t align);

/* -- growable buffers ------------------------------------------------------ */

/* Part XX.1 makes `Array[T]` a compiler-known type until Phase 2's generics
 * let the standard library write it in Ember. Every `Array[T]` has this
 * layout; the compiler emits the element size at each call, which is how one
 * runtime serves every element type.
 *
 * `String` is the same buffer holding UTF-8 bytes, so the two share the growth
 * and free paths rather than each having their own. */
typedef struct ember_vec {
    void* ptr;
    size_t len;
    size_t cap;
} ember_vec;

#define ember_vec_empty() ((ember_vec){ NULL, 0, 0 })

/* Make room for at least `want` elements. Growth doubles, so appending in a
 * loop stays linear ([ALC-1]). */
void ember_vec_reserve(ember_vec* v, size_t elem_size, size_t want);
/* Append one element, copied from `value`. */
void ember_vec_push(ember_vec* v, size_t elem_size, const void* value);
void ember_vec_free(ember_vec* v, size_t elem_size);
/* Append `count` bytes. Used for `String`, whose element size is one. */
void ember_vec_extend(ember_vec* v, const void* bytes, size_t count);

/* A `String`'s bytes as a borrowed `str`. */
ember_str ember_vec_as_str(const ember_vec* v);

/* -- formatting ------------------------------------------------------------ */

/* `[LEX-19]` f-strings append each piece to a buffer. std.fmt's `Display`
 * replaces these once interfaces carry generics; until then the compiler picks
 * one from the argument's type, exactly as it does for `println`. */
void ember_fmt_i64(ember_vec* out, int64_t value);
void ember_fmt_u64(ember_vec* out, uint64_t value);
void ember_fmt_f64(ember_vec* out, double value);
void ember_fmt_f32(ember_vec* out, float value);
void ember_fmt_bool(ember_vec* out, bool value);
void ember_fmt_char(ember_vec* out, uint32_t value);
void ember_fmt_str(ember_vec* out, ember_str value);

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

/* Before `Display` exists the compiler picks a printer from the argument's
 * type. std.fmt replaces these; they stay for the runtime's own diagnostics.
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
 * own systems. */
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

/* Per-thread attach. Idempotent and cheap after the first call ([FFI-22]). */
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
