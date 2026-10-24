/* boxloop.c: a captured variable that lives in a box (A12 rule 2's fallback), written in a hot loop of
   the function that owns it. The Ember program (proposed rules):

       fn main():
           xs = [i % 1000 for i in range(65536)]
           total = 0
           callbacks.push(fn() -> int: total)       # stored and escaping, total written after: a box
           for rep in range(2000):
               for x in xs:
                   total += x
           println(callbacks[0]())

   The report: "The box is one allocation per variable instance and one pointer load per access;
   plain C's callback environment and Rust's Rc<Cell<i64>> pay the same."

     -DC_LOCAL     plain C as a C programmer writes it for speed: a local accumulator, stored once
     -DC_ESCAPED   plain C as written plainly: the callback holds &total, the loop adds into total
     -DEMBER_NAIVE what the rule says is emitted: every access goes through the box pointer; the box
                   comes from the runtime's allocator, which the C compiler cannot see into
     -DEMBER_PROMOTED the save: between calls nothing else can reach the box (a closure runs only
                   inside a call), so the compiler keeps the value in a local across the loop and
                   writes it back before the next call
   Overflow checks are left out of every variant, as in every model of phase 3. */
#include "harness.h"
#define N 65536
#define REPS 2000

typedef struct { uint32_t rc; int64_t value; } Box64;
typedef struct { int64_t (*fn)(void *); void *env; } Callback;
static Callback g_callbacks[4];
static int g_ncb;

/* The runtime's allocator (rtalloc.c, its own translation unit, as ember_alloc is): opaque to the C
   compiler. The Ember variants take both the list's buffer and the box from it, as Ember's runtime does. */
void *rt_alloc(size_t n);

static int64_t read_box(void *env) { return ((Box64 *)env)->value; }
static int64_t read_ptr(void *env) { return *(int64_t *)env; }

int main(void) {
    h_start();
#if defined(C_LOCAL) || defined(C_ESCAPED)
    int64_t *xs = malloc(sizeof(int64_t) * N);
#else
    int64_t *xs = rt_alloc(sizeof(int64_t) * N);
#endif
    for (int64_t i = 0; i < N; i++) xs[i] = i % 1000;
#if defined(C_LOCAL) || defined(C_ESCAPED)
    static int64_t total;            /* the callback reads it through a pointer */
    total = 0;
    g_callbacks[g_ncb++] = (Callback){read_ptr, &total};
#  ifdef C_LOCAL
    int64_t t = 0;
    for (int rep = 0; rep < REPS; rep++)
        for (int64_t i = 0; i < N; i++) t += xs[i];
    total = t;
    const char *name = "boxloop-c-local";
#  else
    for (int rep = 0; rep < REPS; rep++)
        for (int64_t i = 0; i < N; i++) total += xs[i];
    const char *name = "boxloop-c-escaped";
#  endif
#else
    Box64 *box = rt_alloc(sizeof *box);
    box->rc = 1; box->value = 0;
    box->rc++;                       /* the closure's capture */
    g_callbacks[g_ncb++] = (Callback){read_box, box};
#  ifdef EMBER_PROMOTED
    int64_t t = box->value;
    for (int rep = 0; rep < REPS; rep++)
        for (int64_t i = 0; i < N; i++) t += xs[i];
    box->value = t;
    const char *name = "boxloop-ember-promoted";
#  else
    for (int rep = 0; rep < REPS; rep++)
        for (int64_t i = 0; i < N; i++) box->value += xs[i];
    const char *name = "boxloop-ember-naive";
#  endif
#endif
    int64_t r = g_callbacks[0].fn(g_callbacks[0].env);
    h_report(name, (uint64_t)r);
    return 0;
}
