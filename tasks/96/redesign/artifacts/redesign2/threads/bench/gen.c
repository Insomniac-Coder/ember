/* gen.c: a generator that walks the bytes of its own local String and yields each one (A14).
 *
 * Ember (proposed): gen fn letters(n) -> Generator[u8]:
 *                       s = make_text(n)          # a local String: its bytes are heap contents
 *                       for c in s.bytes():       # today E2221; proposed: allowed (heap contents never move)
 *                           yield c
 * Python:           def letters(n):
 *                       s = make_text(n)
 *                       for c in s.encode(): yield c
 *
 * The consumer sums what it gets. Speed: one generator over 1 MiB, run 100 times (104,857,600 yields).
 * Memory (-DMEM): 10,000 generators made and resumed once, all suspended at their first yield at once.
 *
 *   -DMODEL=0  plain C: a loop over the bytes, no generator
 *   -DMODEL=1  the frame Ember compiles a gen fn to ([CORO-4]): a struct with the state, the String and
 *              the iterator's position, and a `next()` that switches on the state; `next()` not inlined
 *   -DMODEL=2  the same frame with `next()` inlined into the consumer's loop
 *   -DMODEL=4  today's workaround (E2221's help: iterate by index): `for i in 0..s.len(): yield s[i]`,
 *              the same frame with the index bounds-checked at the element read; `next()` not inlined
 *   -DMODEL=3  a stackful coroutine instead (the new-mechanism answer): a Win32 fiber per generator,
 *              `yield` is SwitchToFiber back to the consumer (two switches per value)
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n [-DMEM] gen.c -o gen_n.exe */
#include "../../../redesign/bench/harness.h"
#include <string.h>

#ifndef MODEL
#define MODEL 0
#endif
#ifdef MEM
#define GENS 10000
#define LEN 64
#define ROUNDS 1
#else
#define GENS 1
#define LEN (1u << 20)
#define ROUNDS 100
#endif

typedef struct { uint8_t *p; size_t len, cap; } String;
static String make_text(size_t n, uint32_t seed) {
    String s = { (uint8_t *)malloc(n), n, n };
    if (!s.p) abort();
    for (size_t i = 0; i < n; i++) s.p[i] = (uint8_t)('a' + (i * 7 + seed) % 26);
    return s;
}

#if MODEL == 1 || MODEL == 2 || MODEL == 4
/* The frame: parameters, the state, and the locals live across the yield. 40 bytes. */
typedef struct { uint32_t state; uint32_t seed; size_t n; String s; size_t i; } Frame;
#if MODEL == 1 || MODEL == 4
#define NEXT_ATTR __declspec(noinline)
#else
#define NEXT_ATTR __forceinline
#endif
/* Returns 1 and sets *out for a value, 0 at the end. */
static NEXT_ATTR int letters_next(Frame *f, uint8_t *out) {
    switch (f->state) {
    case 0:
        f->s = make_text(f->n, f->seed);
        f->i = 0;
        /* fall through */
    case 1:
        if (f->i < f->s.len) {               /* the loop's own test: i < len; the bytes are on the heap */
#if MODEL == 4
            if (f->i >= f->s.len) { fputs("panic: index out of range\n", stderr); exit(101); }  /* s[i] */
#endif
            *out = f->s.p[f->i++];
            f->state = 1;
            return 1;
        }
        free(f->s.p);
        f->state = 2;
        /* fall through */
    default:
        return 0;
    }
}
#elif MODEL == 3
typedef struct { void *self, *consumer; uint32_t seed; size_t n; uint8_t value; int done; } Fib;
static void WINAPI letters_fiber(void *arg) {
    Fib *g = (Fib *)arg;
    String s = make_text(g->n, g->seed);
    for (size_t i = 0; i < s.len; i++) {
        g->value = s.p[i];
        SwitchToFiber(g->consumer);           /* yield */
    }
    free(s.p);
    g->done = 1;
    for (;;) SwitchToFiber(g->consumer);
}
static int fib_next(Fib *g, uint8_t *out) {
    if (g->done) return 0;
    SwitchToFiber(g->self);
    if (g->done) return 0;
    *out = g->value;
    return 1;
}
#endif

int main(void) {
    h_start();
    uint64_t sum = 0;
#if MODEL == 3
    void *me = ConvertThreadToFiber(NULL);
    if (!me) abort();
#endif
#ifdef MEM
    /* Make GENS generators and resume each once: all of them suspended at once. */
#if MODEL == 1 || MODEL == 2 || MODEL == 4
    Frame *gens = (Frame *)calloc(GENS, sizeof(Frame));
    for (int k = 0; k < GENS; k++) { gens[k].seed = (uint32_t)k; gens[k].n = LEN; uint8_t c; if (letters_next(&gens[k], &c)) sum += c; }
#elif MODEL == 3
    Fib *gens = (Fib *)calloc(GENS, sizeof(Fib));
    for (int k = 0; k < GENS; k++) {
        gens[k].seed = (uint32_t)k; gens[k].n = LEN; gens[k].consumer = me;
        gens[k].self = CreateFiber(0, letters_fiber, &gens[k]);   /* the default stack size of the image */
        if (!gens[k].self) abort();
        uint8_t c; if (fib_next(&gens[k], &c)) sum += c;
    }
#else
    for (int k = 0; k < GENS; k++) { String s = make_text(LEN, (uint32_t)k); sum += s.p[0]; free(s.p); }
#endif
    static const char *names[] = {"gen-mem-0-plainC", "gen-mem-1-frame", "gen-mem-2-frame", "gen-mem-3-fiber",
                                  "gen-mem-4-frame"};
#else
    for (int r = 0; r < ROUNDS; r++) {
#if MODEL == 0
        String s = make_text(LEN, (uint32_t)r);
        for (size_t i = 0; i < s.len; i++) sum += s.p[i];
        free(s.p);
#elif MODEL == 1 || MODEL == 2 || MODEL == 4
        Frame f = {0};
        f.seed = (uint32_t)r; f.n = LEN;
        uint8_t c;
        while (letters_next(&f, &c)) sum += c;
#else
        Fib g = {0};
        g.seed = (uint32_t)r; g.n = LEN; g.consumer = me;
        g.self = CreateFiber(0, letters_fiber, &g);
        if (!g.self) abort();
        uint8_t c;
        while (fib_next(&g, &c)) sum += c;
        DeleteFiber(g.self);
#endif
    }
    static const char *names[] = {"gen-0-plainC", "gen-1-frame-call", "gen-2-frame-inlined", "gen-3-fiber",
                                  "gen-4-today-index-workaround"};
#endif
    h_report(names[MODEL], sum);
    return 0;
}
