/* cb_parse_lib.h: workload cb_parse's C library and its text, shared by every C program of the
   workload (the library is C code, the same in each; only the callbacks differ). */
#pragma once
#include "m4.h"

#define FIELDS 2000000
#define PER_ROW 8
#define PASSES 10

typedef void (*field_cb)(void *ud, int64_t value);
typedef void (*row_cb)(void *ud);

/* The C library. */
static NOINLINE void csv_parse(const char *text, size_t n, field_cb on_field, void *field_ud, row_cb on_row, void *row_ud) {
    int64_t v = 0;
    for (size_t i = 0; i < n; i++) {
        char c = text[i];
        if (c >= '0' && c <= '9') v = v * 10 + (c - '0');
        else if (c == ',') { on_field(field_ud, v); v = 0; }
        else if (c == '\n') { on_field(field_ud, v); v = 0; on_row(row_ud); }
    }
}

/* The text every program of the workload parses (the same bytes in each). */
static char *make_text(size_t *len) {
    char *t = (char *)malloc((size_t)FIELDS * 7);
    size_t at = 0;
    h_seed(41);
    for (int i = 0; i < FIELDS; i++) {
        uint32_t v = h_rand() % 100000;
        char digits[8]; int nd = 0;
        do { digits[nd++] = (char)('0' + v % 10); v /= 10; } while (v);
        while (nd) t[at++] = digits[--nd];
        t[at++] = (i % PER_ROW == PER_ROW - 1) ? '\n' : ',';
    }
    *len = at;
    return t;
}

