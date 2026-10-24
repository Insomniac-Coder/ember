/* keep.c: A22 6.5, "a view of a temporary returned as an owned String reuses the temporary's buffer".
   first_word(line) = line.to_lower() (a new buffer of the line's length), then the first space-separated word
   after the leading spaces, returned as an owned String and kept by the caller (firsts.push(first_word(line))).
     -DREUSE  : 6.5 as written: move the word to the front of the lowered buffer, set the length, return that
                buffer with its whole capacity.
     -DCOPY   : today's `.to_string()`, and what Python makes: an exact-size copy; the lowered buffer is freed.
     -DSHRINK : a first try at saving 6.5: as REUSE, then give the unused tail back with realloc when more than
                half of the buffer and more than 64 bytes are unused.
     -DHALF   : 6.5 saved: reuse the buffer only when the result fills at least half of it; otherwise make the
                exact-size copy and free the buffer (so a kept result never holds more than twice its length).
     -DTRIM   : the result is the whole lowered line without its leading and trailing spaces (`.trim()`),
                instead of its first word.
     -DLONGLINES: 200 lines of 1 MiB each, generated one at a time into one input buffer (the memory case);
                otherwise 2,000,000 lines of 8..80 bytes generated up front (the time case). */
#include "harness.h"
#include <string.h>
typedef struct { char *p; size_t len, cap; } Str;
static inline Str first_word(const char *line, size_t n) {
    char *b = (char *)malloc(n ? n : 1);
    for (size_t i = 0; i < n; i++) { char c = line[i]; b[i] = (c >= 'A' && c <= 'Z') ? (char)(c + 32) : c; }
    size_t s = 0; while (s < n && b[s] == ' ') s++;
#ifdef TRIM
    size_t e = n; while (e > s && b[e - 1] == ' ') e--;
#else
    size_t e = s; while (e < n && b[e] != ' ') e++;
#endif
    size_t k = e - s;
#if defined(HALF)
    if (2 * k < n) { char *c = (char *)malloc(k ? k : 1); memcpy(c, b + s, k); free(b); return (Str){c, k, k}; }
    memmove(b, b + s, k);
    return (Str){b, k, n};
#elif defined(COPY)
    char *c = (char *)malloc(k ? k : 1); memcpy(c, b + s, k); free(b);
    return (Str){c, k, k};
#else
    memmove(b, b + s, k);
#if defined(SHRINK)
    if (n - k > 64 && n - k > n / 2) { char *r = (char *)realloc(b, k ? k : 1); if (r) b = r; return (Str){b, k, k}; }
#endif
    return (Str){b, k, n};
#endif
}
static inline void gen_line(char *t, size_t n) {
    size_t lead = h_rand() % 4, i = 0;
    for (; i < lead && i < n; i++) t[i] = ' ';
    while (i < n) {
        size_t w = 1 + h_rand() % 9;
        for (size_t j = 0; j < w && i < n; j++) t[i++] = (char)((h_rand() % 2 ? 'A' : 'a') + h_rand() % 26);
        if (i < n) t[i++] = ' ';
    }
}
#ifdef LONGLINES
#define NL 200u
#else
#define NL 2000000u
#endif
int main(void) {
    h_start(); h_seed(5);
    Str *firsts = (Str *)malloc(NL * sizeof(Str));
    LARGE_INTEGER w0, w1, f;
#ifdef LONGLINES
    size_t L = (size_t)1 << 20; char *line = (char *)malloc(L);
    QueryPerformanceCounter(&w0);
    for (uint32_t i = 0; i < NL; i++) { gen_line(line, L); firsts[i] = first_word(line, L); }
#else
    size_t *off = (size_t *)malloc((NL + 1) * sizeof(size_t)); size_t tot = 0;
    for (uint32_t i = 0; i < NL; i++) { off[i] = tot; tot += 8 + h_rand() % 73; }
    off[NL] = tot; char *text = (char *)malloc(tot);
    for (uint32_t i = 0; i < NL; i++) gen_line(text + off[i], off[i + 1] - off[i]);
    QueryPerformanceCounter(&w0);
    for (uint32_t i = 0; i < NL; i++) firsts[i] = first_word(text + off[i], off[i + 1] - off[i]);
#endif
    uint64_t sum = 0, kept = 0;
    for (uint32_t i = 0; i < NL; i++) { sum += firsts[i].len * 31 + (firsts[i].len ? (uint8_t)firsts[i].p[0] : 0); kept += firsts[i].cap; }
    PROCESS_MEMORY_COUNTERS pmc; GetProcessMemoryInfo(GetCurrentProcess(), &pmc, sizeof pmc);
    size_t ws_kb = pmc.WorkingSetSize / 1024;
    for (uint32_t i = 0; i < NL; i++) free(firsts[i].p);
    QueryPerformanceCounter(&w1); QueryPerformanceFrequency(&f);
    printf("work_ms=%.3f kept_bytes=%llu ws_before_free_kb=%llu ", (double)(w1.QuadPart - w0.QuadPart) * 1000.0 / (double)f.QuadPart,
           (unsigned long long)kept, (unsigned long long)ws_kb);
#if defined(COPY)
    const char *v = "copy";
#elif defined(SHRINK)
    const char *v = "shrink";
#elif defined(HALF)
    const char *v = "half";
#else
    const char *v = "reuse";
#endif
    char name[32];
#if defined(LONGLINES)
    snprintf(name, sizeof name, "keep-%s-long", v);
#elif defined(TRIM)
    snprintf(name, sizeof name, "keep-%s-trim", v);
#else
    snprintf(name, sizeof name, "keep-%s-short", v);
#endif
    h_report(name, sum);
    return 0;
}
