/* words_common.h: the A7/A22/A6 workload shared by every C model.
   NW words of 1..12 lowercase letters, one space between words (none at the end), all from h_rand().
   Work = split the text into NW stored words, PASSES in-order passes hashing every word's bytes,
   NRAND random word reads, then drop everything. "work_ms" times exactly that (text generation excluded). */
#pragma once
#include "harness.h"
#include <string.h>
#define NW 4000000u
#define PASSES 10
#define NRAND 8000000u

static char *make_text_into(char *t, size_t *len_out) {
    size_t k = 0;
    for (uint32_t i = 0; i < NW; i++) {
        if (i) t[k++] = ' ';
        uint32_t n = 1 + h_rand() % 12;
        for (uint32_t j = 0; j < n; j++) t[k++] = (char)('a' + h_rand() % 26);
    }
    *len_out = k;
    return t;
}
static inline uint64_t fnv(const char *p, size_t n) {
    uint64_t h = 1469598103934665603ull;
    for (size_t i = 0; i < n; i++) { h ^= (uint8_t)p[i]; h *= 1099511628211ull; }
    return h;
}
static LARGE_INTEGER w_t0;
static inline void work_start(void) { QueryPerformanceCounter(&w_t0); }
static inline void work_end(void) {
    LARGE_INTEGER t1, f; QueryPerformanceCounter(&t1); QueryPerformanceFrequency(&f);
    printf("work_ms=%.3f ", (double)(t1.QuadPart - w_t0.QuadPart) * 1000.0 / (double)f.QuadPart);
}
