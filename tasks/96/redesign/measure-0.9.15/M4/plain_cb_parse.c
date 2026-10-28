/* plain_cb_parse.c: workload cb_parse, plain C. A callback-heavy C library in the style of SAX and
   libcsv parsers: `csv_parse(text, n, on_field, field_ud, on_row, row_ud)`, not inlined, walks a text
   of comma-separated numbers and calls `on_field(ud, value)` for each number and `on_row(ud)` at each
   line's end. The caller's callbacks sum the values, count them and keep the maximum (in `stats`), and
   count the rows (in `rows`).
   The text: 2,000,000 numbers in 0 .. 99,999, 8 per line (11,777,043 bytes); parsed 10 times: 20,000,000
   field calls and 2,500,000 row calls. The callbacks are read from volatiles, so the C compiler cannot
   see which functions the library calls. The user data is `&stats` and `&rows`. */
#include "cb_parse_lib.h"

typedef struct { int64_t sum, count, max; } Stats;
static void on_field_c(void *ud, int64_t v) {
    Stats *s = (Stats *)ud;
    s->sum += v; s->count += 1;
    if (v > s->max) s->max = v;
}
static void on_row_c(void *ud) { *(int64_t *)ud += 1; }

static field_cb volatile the_field = on_field_c;
static row_cb volatile the_row = on_row_c;

int main(void) {
    h_start();
    size_t len;
    char *text = make_text(&len);
    Stats st = {0, 0, 0};
    int64_t rows = 0;
    for (int p = 0; p < PASSES; p++) csv_parse(text, len, the_field, &st, the_row, &rows);
    h_report("plain_cb_parse", (uint64_t)st.sum + (uint64_t)st.count * 7u + (uint64_t)st.max * 13u + (uint64_t)rows * 17u);
    return 0;
}
