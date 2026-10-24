/* A3 attack: a view of one name's list passed into a write through the other name.
       fn merge(mut a: Squad, b: Squad):          def merge(a, b):
           a.units.extend(b.units[0..2])              a.units.extend(b.units[0:2])
       merge(squads[i], squads[j])                merge(squads[i], squads[j])
   A3 rule 1 allows i == j (one squad, two names). Rule 2 forbids keeping a view of b.units' elements
   *after* a write through a that may replace them; here the view is used *during* the write, inside
   extend. extend is written the usual way (Rust's extend_from_slice): make room, then copy. When a and
   b are one squad, making room frees the buffer the view points into.
   A poisoning allocator fills a freed block with 0xDD and keeps it, so the copy shows what it read.
   Python (probes/py_attack_output.txt): ys.extend(ys[0:2]) gives [1, 2, 3, 1, 2]. */
#include "../harness.h"
#include <string.h>
typedef struct { uint64_t magic, size; } Hdr;
static void *xmalloc(size_t n) { Hdr *h = malloc(sizeof(Hdr) + n); h->magic = 1; h->size = n; return h + 1; }
static void xfree(void *p) { Hdr *h = (Hdr *)p - 1; h->magic = 0; memset(p, 0xDD, h->size); }  /* kept, poisoned */
typedef struct { int32_t *data; size_t len, cap; } List;
typedef struct { const int32_t *p; size_t n; } Span;
static void reserve(List *l, size_t extra) {
    if (l->len + extra <= l->cap) return;
    size_t c = l->cap ? l->cap : 4; while (c < l->len + extra) c *= 2;
    int32_t *d = xmalloc(c * 4); memcpy(d, l->data, l->len * 4); xfree(l->data); l->data = d; l->cap = c;
}
static void extend(List *l, Span s) {                      /* the library function, as usual */
    reserve(l, s.n);
    memcpy(l->data + l->len, s.p, s.n * 4); l->len += s.n;
}
static void extend_call_saved(List *l, Span s) {           /* FIX at the call: one range compare */
    int32_t tmp[64];
    if ((uintptr_t)s.p - (uintptr_t)l->data < l->cap * 4) {  /* the view points into l's buffer */
        memcpy(tmp, s.p, s.n * 4); s.p = tmp;              /* copy it first: Python's slice is a copy */
    }
    extend(l, s);
}
static void show(const char *what, const List *l) {
    printf("%-34s [", what);
    for (size_t k = 0; k < l->len; k++) printf(k ? ", %d" : "%d", l->data[k]);
    printf("]\n");
}
int main(void) {
    h_start();
    for (int saved = 0; saved < 2; saved++) {
        List units = {xmalloc(3 * 4), 3, 3};
        units.data[0] = 1; units.data[1] = 2; units.data[2] = 3;
        List *a = &units, *b = &units;                     /* merge(squads[i], squads[i]) */
        Span view = {b->data, 2};                          /* b.units[0..2] */
        if (saved) extend_call_saved(a, view); else extend(a, view);
        show(saved ? "saved (Python: [1, 2, 3, 1, 2]):" : "as written (reads freed memory):", &units);
    }
    return 0;
}
