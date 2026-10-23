/* phase 2 (Lobster / Koka-Perceus / Roc / Nim ARC-ORC): what reference counting costs against plain C.
   Every model holds the same 4,194,304 16-byte components.
   - inline_*: plain C, values in one contiguous array (what Ember's Array[T] of structs is today).
   - boxed_*:  every element its own heap object with an 8-byte count header, the array holds pointers
               (a Lobster `class`, a Nim `ref object`, a Koka constructor, a Swift class).
       boxed_iter          the count operations elided (a borrow): only the indirection and the header
       boxed_iter_shuffled the same, the pointer array shuffled (a heap after churn)
       boxed_iter_rc       one non-atomic dup + drop per element, as when elision fails (Koka's fast path)
       boxed_iter_atomic   one atomic increment + decrement per element (Swift, Nim atomicArc, Rust Arc)
   - inline_set / unique_set / cow_first_set: random writes into a list; unique_set checks the list's
     count before every write as Roc's List.set and Koka's reuse do; cow_first_set times ONE write into a
     list that is shared (count 2), which copies all 64 MiB first.
   One mode per process so peak_kb belongs to that mode; work_ms times the phase after the fill.
   Build: clang -O2 rc_models.c -o rc_models.exe   Run: rc_models.exe <mode> */
#include "../bench/harness.h"
#include <string.h>

typedef struct { int64_t a, b; } Comp;
typedef struct { int64_t rc; Comp c; } Obj;                          /* 8-byte header + payload */
typedef struct { int64_t rc; uint32_t len; Comp data[]; } List;      /* one header for the whole list */
#define N      (1u << 22)
#define PASSES 20
#define HITS   (1u << 24)
#define BARRIER() __asm__ volatile("" ::: "memory")   /* the borrowed value used across opaque code */

__declspec(noinline) static void drop_slow(Obj *o) {      /* count reached zero: free (never here) */
    if (o->rc <= 1) { free(o); } else o->rc--;
}
static inline void dup(Obj *o) { o->rc++; }
static inline void drop(Obj *o) { if (o->rc <= 1) drop_slow(o); else o->rc--; }   /* Perceus 2.7.2 */

__declspec(noinline) static List *list_copy(List *l) {
    size_t bytes = sizeof(List) + (size_t)l->len * sizeof(Comp);
    List *n = malloc(bytes); memcpy(n, l, bytes); n->rc = 1; l->rc--; return n;
}

int main(int argc, char **argv) {
    const char *m = argc > 1 ? argv[1] : "";
    h_start(); h_seed(42);
    uint64_t sum = 0;
    LARGE_INTEGER tw, te, fq;
    int boxed = !strncmp(m, "boxed", 5), listy = !strcmp(m, "unique_set") || !strcmp(m, "cow_first_set");
    Comp *dense = NULL; Obj **objs = NULL; List *list = NULL;
    if (boxed) {
        objs = malloc(N * sizeof(Obj *));
        for (uint32_t i = 0; i < N; i++) { Obj *o = malloc(sizeof(Obj)); o->rc = 1; o->c.a = i; o->c.b = h_rand(); objs[i] = o; }
        if (!strcmp(m, "boxed_iter_shuffled"))
            for (uint32_t i = N - 1; i > 0; i--) { uint32_t j = h_rand() % (i + 1); Obj *t = objs[i]; objs[i] = objs[j]; objs[j] = t; }
    } else if (listy) {
        list = malloc(sizeof(List) + (size_t)N * sizeof(Comp)); list->rc = 1; list->len = N;
        for (uint32_t i = 0; i < N; i++) { list->data[i].a = i; list->data[i].b = h_rand(); }
    } else {
        dense = malloc((size_t)N * sizeof(Comp));
        for (uint32_t i = 0; i < N; i++) { dense[i].a = i; dense[i].b = h_rand(); }
    }
    QueryPerformanceCounter(&tw);
    if (!strcmp(m, "inline_iter")) {
        for (int p = 0; p < PASSES; p++) for (uint32_t i = 0; i < N; i++) sum += dense[i].a + dense[i].b;
    } else if (!strcmp(m, "boxed_iter") || !strcmp(m, "boxed_iter_shuffled")) {
        for (int p = 0; p < PASSES; p++) for (uint32_t i = 0; i < N; i++) sum += objs[i]->c.a + objs[i]->c.b;
    } else if (!strcmp(m, "boxed_iter_rc")) {
        for (int p = 0; p < PASSES; p++) for (uint32_t i = 0; i < N; i++) {
            Obj *o = objs[i]; dup(o); BARRIER(); sum += o->c.a + o->c.b; BARRIER(); drop(o);
        }
    } else if (!strcmp(m, "boxed_iter_atomic")) {
        for (int p = 0; p < PASSES; p++) for (uint32_t i = 0; i < N; i++) {
            Obj *o = objs[i];
            __atomic_fetch_add(&o->rc, 1, __ATOMIC_RELAXED); BARRIER();
            sum += o->c.a + o->c.b; BARRIER();
            if (__atomic_fetch_sub(&o->rc, 1, __ATOMIC_ACQ_REL) == 1) free(o);
        }
    } else if (!strcmp(m, "inline_set")) {
        for (uint32_t k = 0; k < HITS; k++) { uint32_t i = h_rand() & (N - 1); dense[i].a += 1; }
        for (uint32_t i = 0; i < N; i++) sum += dense[i].a;
    } else if (!strcmp(m, "unique_set")) {
        for (uint32_t k = 0; k < HITS; k++) {
            uint32_t i = h_rand() & (N - 1);
            if (list->rc != 1) list = list_copy(list);          /* the is-unique test before each write */
            list->data[i].a += 1;
        }
        for (uint32_t i = 0; i < N; i++) sum += list->data[i].a;
    } else if (!strcmp(m, "cow_first_set")) {
        list->rc = 2;                                           /* a second owner exists */
        QueryPerformanceCounter(&tw);
        if (list->rc != 1) list = list_copy(list);
        list->data[7].a += 1;
        QueryPerformanceCounter(&te);
        sum = (uint64_t)list->data[7].a;
        QueryPerformanceFrequency(&fq);
        printf("%s work_ms=%.3f ", m, (double)(te.QuadPart - tw.QuadPart) * 1000.0 / (double)fq.QuadPart);
        h_report(m, sum);
        return 0;
    } else { fputs("modes: inline_iter boxed_iter boxed_iter_shuffled boxed_iter_rc boxed_iter_atomic inline_set unique_set cow_first_set\n", stderr); return 2; }
    QueryPerformanceCounter(&te); QueryPerformanceFrequency(&fq);
    printf("%s work_ms=%.3f ", m, (double)(te.QuadPart - tw.QuadPart) * 1000.0 / (double)fq.QuadPart);
    h_report(m, sum);
    return 0;
}
