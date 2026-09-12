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

/* -- arenas -----------------------------------------------------------------
 *
 * Chunks never move: growing an arena appends a chunk rather than reallocating
 * an existing one, so every ref/MutSpan already handed out keeps its address
 * ([ARN-1], [LT-4]). Each data allocation records its own alignment for the
 * allocator's matching free call. A reset retains chunks and rewinds them;
 * the borrow checker has already proved that no returned view is live
 * ([ARN-7]). */

typedef struct ember_arena_chunk {
    struct ember_arena_chunk* next;
    unsigned char* data;
    size_t capacity;
    size_t used;
    size_t align;
} ember_arena_chunk;

typedef struct ember_arena_state {
    ember_arena_chunk* first;
    ember_arena_chunk* current;
} ember_arena_state;

#define EMBER_ARENA_GROWTH_CHUNK ((size_t)1024 * (size_t)1024)

static bool arena_power_of_two(size_t value) {
    return value != 0 && (value & (value - 1)) == 0;
}

static ember_arena_chunk* arena_chunk_new(size_t capacity, size_t align) {
    ember_arena_chunk* chunk = (ember_arena_chunk*)ember_alloc(
        sizeof(ember_arena_chunk), _Alignof(ember_arena_chunk));
    chunk->next = NULL;
    chunk->capacity = capacity;
    chunk->used = 0;
    chunk->align = align;
    chunk->data = (unsigned char*)ember_alloc(capacity, align);
    return chunk;
}

void* ember_arena_new(size_t initial_capacity) {
    ember_arena_state* arena = (ember_arena_state*)ember_alloc(
        sizeof(ember_arena_state), _Alignof(ember_arena_state));
    arena->first = NULL;
    arena->current = NULL;
    if (initial_capacity != 0) {
        size_t align = _Alignof(max_align_t);
        arena->first = arena_chunk_new(initial_capacity, align);
        arena->current = arena->first;
    }
    return arena;
}

static void* arena_chunk_take(ember_arena_chunk* chunk, size_t size, size_t align) {
    if (chunk->align < align || chunk->used > SIZE_MAX - (align - 1)) {
        return NULL;
    }
    size_t offset = (chunk->used + align - 1) & ~(align - 1);
    if (offset > chunk->capacity || size > chunk->capacity - offset) {
        return NULL;
    }
    chunk->used = offset + size;
    return chunk->data + offset;
}

void* ember_arena_alloc_copy(void* opaque, size_t size, size_t align, const void* value) {
    ember_arena_state* arena = (ember_arena_state*)opaque;
    if (arena == NULL || value == NULL || !arena_power_of_two(align)) {
        ember_loc loc = { "<arena>", 0, 0 };
        ember_panic("invalid arena allocation", 24, loc);
    }
    if (size == 0) {
        size = 1;
    }

    ember_arena_chunk* chunk = arena->current ? arena->current : arena->first;
    while (chunk != NULL) {
        void* slot = arena_chunk_take(chunk, size, align);
        if (slot != NULL) {
            arena->current = chunk;
            memcpy(slot, value, size);
            return slot;
        }
        chunk = chunk->next;
    }

    size_t capacity = size > EMBER_ARENA_GROWTH_CHUNK ? size : EMBER_ARENA_GROWTH_CHUNK;
    size_t chunk_align = align > _Alignof(max_align_t) ? align : _Alignof(max_align_t);
    ember_arena_chunk* fresh = arena_chunk_new(capacity, chunk_align);
    if (arena->first == NULL) {
        arena->first = fresh;
    } else {
        ember_arena_chunk* tail = arena->first;
        while (tail->next != NULL) {
            tail = tail->next;
        }
        tail->next = fresh;
    }
    arena->current = fresh;
    void* slot = arena_chunk_take(fresh, size, align);
    if (slot == NULL) {
        ember_loc loc = { "<arena>", 0, 0 };
        ember_panic("arena allocation overflow", 25, loc);
    }
    memcpy(slot, value, size);
    return slot;
}

void ember_arena_reset(void* opaque) {
    ember_arena_state* arena = (ember_arena_state*)opaque;
    if (arena == NULL) {
        return;
    }
    for (ember_arena_chunk* chunk = arena->first; chunk != NULL; chunk = chunk->next) {
        chunk->used = 0;
    }
    arena->current = arena->first;
}

void* ember_arena_mark(void* opaque) {
    ember_arena_state* arena = (ember_arena_state*)opaque;
    if (arena == NULL || arena->current == NULL) {
        return NULL;
    }
    return arena->current->data + arena->current->used;
}

void ember_arena_rewind(void* opaque, void* mark) {
    ember_arena_state* arena = (ember_arena_state*)opaque;
    if (arena == NULL) {
        return;
    }
    if (mark == NULL) {
        ember_arena_reset(arena);
        return;
    }
    uintptr_t wanted = (uintptr_t)mark;
    for (ember_arena_chunk* chunk = arena->first; chunk != NULL; chunk = chunk->next) {
        uintptr_t start = (uintptr_t)chunk->data;
        if (start <= wanted && wanted - start <= chunk->capacity) {
            chunk->used = (size_t)(wanted - start);
            for (ember_arena_chunk* later = chunk->next; later != NULL; later = later->next) {
                later->used = 0;
            }
            arena->current = chunk;
            return;
        }
    }
    ember_loc loc = { "<arena>", 0, 0 };
    ember_panic("invalid arena mark", 18, loc);
}

void ember_arena_free(void* opaque) {
    ember_arena_state* arena = (ember_arena_state*)opaque;
    if (arena == NULL) {
        return;
    }
    ember_arena_chunk* chunk = arena->first;
    while (chunk != NULL) {
        ember_arena_chunk* next = chunk->next;
        ember_free(chunk->data, chunk->capacity, chunk->align);
        ember_free(chunk, sizeof(ember_arena_chunk), _Alignof(ember_arena_chunk));
        chunk = next;
    }
    ember_free(arena, sizeof(ember_arena_state), _Alignof(ember_arena_state));
}

void* ember_fixed_arena_alloc_copy(
    void* buffer,
    size_t capacity,
    size_t* used,
    size_t size,
    size_t align,
    const void* value
) {
    if ((buffer == NULL && capacity != 0) || used == NULL || value == NULL ||
        !arena_power_of_two(align)) {
        ember_loc loc = { "<fixed arena>", 0, 0 };
        ember_panic("invalid fixed arena allocation", 30, loc);
    }
    if (size == 0) {
        size = 1;
    }
    if (capacity == 0) {
        ember_loc loc = { "<fixed arena>", 0, 0 };
        ember_panic("fixed arena exhausted", 21, loc);
    }
    uintptr_t base = (uintptr_t)buffer;
    if (*used > UINTPTR_MAX - base || base + *used > UINTPTR_MAX - (align - 1)) {
        ember_loc loc = { "<fixed arena>", 0, 0 };
        ember_panic("fixed arena allocation overflow", 31, loc);
    }
    uintptr_t aligned = (base + *used + align - 1) & ~(uintptr_t)(align - 1);
    size_t offset = (size_t)(aligned - base);
    if (offset > capacity || size > capacity - offset) {
        ember_loc loc = { "<fixed arena>", 0, 0 };
        ember_panic("fixed arena exhausted", 21, loc);
    }
    *used = offset + size;
    void* slot = (void*)aligned;
    memcpy(slot, value, size);
    return slot;
}

/* -- growable buffers -------------------------------------------------------- */

/* The alignment a growable buffer allocates at. The compiler passes an element
 * size but not an alignment, so the buffer uses the strictest fundamental one;
 * over-aligning is always sound and costs at most a few bytes per buffer. */
#define EMBER_VEC_ALIGN (sizeof(void*) * 2)

void ember_vec_reserve(ember_vec* v, size_t elem_size, size_t want) {
    if (want <= v->cap) {
        return;
    }
    /* Doubling keeps a loop of pushes linear ([ALC-1]); the floor of four
     * stops a one-element array from reallocating on its second push. */
    size_t cap = v->cap < 4 ? 4 : v->cap;
    while (cap < want) {
        cap *= 2;
    }
    v->ptr = ember_realloc(v->ptr, v->cap * elem_size, cap * elem_size, EMBER_VEC_ALIGN);
    v->cap = cap;
}

void ember_vec_push(ember_vec* v, size_t elem_size, const void* value) {
    ember_vec_reserve(v, elem_size, v->len + 1);
    memcpy((unsigned char*)v->ptr + v->len * elem_size, value, elem_size);
    v->len += 1;
}

void ember_vec_extend(ember_vec* v, const void* bytes, size_t count) {
    if (count == 0) {
        return;
    }
    ember_vec_reserve(v, 1, v->len + count);
    memcpy((unsigned char*)v->ptr + v->len, bytes, count);
    v->len += count;
}

void ember_vec_free(ember_vec* v, size_t elem_size) {
    if (v->ptr != NULL) {
        ember_free(v->ptr, v->cap * elem_size, EMBER_VEC_ALIGN);
    }
    v->ptr = NULL;
    v->len = 0;
    v->cap = 0;
}

ember_str ember_vec_as_str(const ember_vec* v) {
    ember_str s;
    s.ptr = (const unsigned char*)v->ptr;
    s.len = v->len;
    return s;
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

void ember_panic_refcell(const char* file, uint32_t line, ember_loc loc) {
    char buffer[512];
    int written;
    if (file != NULL) {
        written = snprintf(buffer, sizeof buffer,
                           "RefCell already mutably borrowed (borrowed at %s:%u)",
                           file, (unsigned)line);
    } else {
        written = snprintf(buffer, sizeof buffer,
                           "RefCell already mutably borrowed (borrowed at unknown:0)");
    }
    panic_with(buffer, written > 0 ? (size_t)written : 0, loc);
}

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

/* The shortest text that reads back as the same value.
 *
 * Taking the first precision that round-trips is not the same thing: `%.1g`
 * of 10.0 is "1e+01", which round-trips exactly, so a loop that stops there
 * prints 10.0 as "1e+01". Every precision is tried and the shortest result
 * kept, which picks "10" here and still picks "1e+20" over twenty-one
 * digits. */
static void write_shortest_f64(double v, char* buffer, size_t cap) {
    char candidate[32];
    buffer[0] = '\0';
    for (int precision = 1; precision <= 17; ++precision) {
        snprintf(candidate, sizeof candidate, "%.*g", precision, v);
        if (strtod(candidate, NULL) != v) {
            continue;
        }
        if (buffer[0] == '\0' || strlen(candidate) < strlen(buffer)) {
            snprintf(buffer, cap, "%s", candidate);
        }
    }
    if (buffer[0] == '\0') {
        snprintf(buffer, cap, "%.17g", v);
    }
}

static void write_shortest_f32(float v, char* buffer, size_t cap) {
    char candidate[32];
    buffer[0] = '\0';
    for (int precision = 1; precision <= 9; ++precision) {
        snprintf(candidate, sizeof candidate, "%.*g", precision, (double)v);
        if (strtof(candidate, NULL) != v) {
            continue;
        }
        if (buffer[0] == '\0' || strlen(candidate) < strlen(buffer)) {
            snprintf(buffer, cap, "%s", candidate);
        }
    }
    if (buffer[0] == '\0') {
        snprintf(buffer, cap, "%.9g", (double)v);
    }
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
/* -- formatting -------------------------------------------------------------- */

void ember_fmt_i64(ember_vec* out, int64_t value) {
    char buffer[32];
    int n = snprintf(buffer, sizeof buffer, "%lld", (long long)value);
    if (n > 0) {
        ember_vec_extend(out, buffer, (size_t)n);
    }
}

void ember_fmt_u64(ember_vec* out, uint64_t value) {
    char buffer[32];
    int n = snprintf(buffer, sizeof buffer, "%llu", (unsigned long long)value);
    if (n > 0) {
        ember_vec_extend(out, buffer, (size_t)n);
    }
}

void ember_fmt_f64(ember_vec* out, double value) {
    char buffer[32];
    write_shortest_f64(value, buffer, sizeof buffer);
    ember_vec_extend(out, buffer, strlen(buffer));
}

void ember_fmt_f32(ember_vec* out, float value) {
    char buffer[32];
    write_shortest_f32(value, buffer, sizeof buffer);
    ember_vec_extend(out, buffer, strlen(buffer));
}

void ember_fmt_bool(ember_vec* out, bool value) {
    const char* text = value ? "true" : "false";
    ember_vec_extend(out, text, strlen(text));
}

void ember_fmt_char(ember_vec* out, uint32_t value) {
    unsigned char buffer[4];
    size_t n = encode_utf8(value, buffer);
    ember_vec_extend(out, buffer, n);
}

void ember_fmt_str(ember_vec* out, ember_str value) {
    ember_vec_extend(out, value.ptr, value.len);
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
