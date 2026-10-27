/* model_cb_parse.c: workload cb_parse, what Ember emits.

       struct Stats:
           sum: int = 0
           count: int = 0
           max: int = 0

       unsafe extern "C":
           @ffi(param(on_field, callback(borrowed, user_data=field_ud), threads(creator)),
                param(on_row, callback(borrowed, user_data=row_ud), threads(creator)))
           safe fn csv_parse(text: Span[u8], on_field: extern "C" fn(*mut void, i64), field_ud: *mut void,
                             on_row: extern "C" fn(*mut void), row_ud: *mut void)

       fn main():
           text = make_text()
           stats = Stats()
           rows = 0
           for p in 0..10:
               csv_parse(text.as_span(), fn(v: i64): stats.sum += v; stats.count += 1; if v > stats.max: stats.max = v,
                         fn(): rows += 1)

   Two `borrowed` callbacks: C runs them only during the call, so each uses the caller's real variable
   with no capture word (R12); they change different variables, so R4 allows them in one call. Their
   contract says `threads(creator)`, as the stats are this thread's.

   Per call of `csv_parse`: each callback is registered in a slot of the table (its state: the address
   of the real variable it changes), C is given the trampolines and the tagged numbers, and after the
   call both registrations end (the next generation: a number C kept past the call is stale).
   Per callback call, the checked trampoline (cb_rt.h; [FFI-53], R15): attach ([FFI-22]); the slot inside
   the table; generation and type in one compare; the creator thread in every profile ([FFI-53],
   [FFI-33b]). The bodies' `+=` are checked ([TYP-8]).
   -DNO_THREAD_CHECK: without the thread compare (today's release profile).
   salvage_cb_parse_fold.c and salvage_cb_parse_teb.c include this file with their mode. */
#include "cb_rt.h"
#include "cb_parse_lib.h"

#define TYPE_FIELD 21u
#define TYPE_ROW 22u

#ifndef NAME
#ifdef NO_THREAD_CHECK
#define NAME "model_cb_parse_nothread"
#else
#define NAME "model_cb_parse"
#endif
#endif

typedef struct { int64_t sum, count, max; } Stats;

static void tramp_field(void *ud, int64_t v) {
    Slot *s = cb_enter(ud, TYPE_FIELD);
    Stats *st = (Stats *)s->env[0];                             /* the real variable `stats` */
    st->sum = ck_add(st->sum, v);                               /* [TYP-8] */
    st->count = ck_add(st->count, 1);                           /* [TYP-8] */
    if (v > st->max) st->max = v;
}
static void tramp_row(void *ud) {
    Slot *s = cb_enter(ud, TYPE_ROW);
    int64_t *rows = (int64_t *)s->env[0];                       /* the real variable `rows` */
    *rows = ck_add(*rows, 1);                                   /* [TYP-8] */
}

static field_cb volatile the_field = tramp_field;
static row_cb volatile the_row = tramp_row;

int main(void) {
    h_start();
    cb_init(64);
    size_t len;
    char *text = make_text(&len);
    Stats st = {0, 0, 0};
    int64_t rows = 0;
    for (int p = 0; p < PASSES; p++) {
        uint64_t hf = cb_register(TYPE_FIELD, 0, &st, 0);      /* the call's registrations */
        uint64_t hr = cb_register(TYPE_ROW, 0, &rows, 0);
        csv_parse(text, len, the_field, (void *)(uintptr_t)hf, the_row, (void *)(uintptr_t)hr);
        cb_end(hf & SLOT_MASK);                                 /* the call returned: both end */
        cb_end(hr & SLOT_MASK);
    }
    h_report(NAME, (uint64_t)st.sum + (uint64_t)st.count * 7u + (uint64_t)st.max * 13u + (uint64_t)rows * 17u);
    return 0;
}
