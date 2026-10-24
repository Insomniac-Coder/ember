/* A3 memory attacks on the first attack's saved rules (phase3b-containers-attack.md, section 9, A3):
     rule 3: "in the alias copy, a write that replaces a heap part another view still uses keeps the
              old buffer alive until the view's last use"
     rule 4: "a call that changes a list and receives a view that may point into it compares the view's
              range with the buffer and copies the view first when it is inside"
   A poisoning allocator fills a freed block with 0xDD and keeps it (never handed back), so reading
   freed memory shows 0xDDDDDDDD (-572662307) or a dead block header, and nothing crashes.

   Demo R (reinforce): a view passed with the list of squads to a function that grows every squad.
       def reinforce(squads, picked):            fn reinforce(mut squads: Array[Squad], picked: Span[int]):
           for s in squads:                          for s in squads:
               s.units.extend(picked)                    s.units.extend(picked)
       reinforce(squads, squads[0].units[0:2])   reinforce(squads, squads[0].units[0..2])
     Rule 4 runs inside each extend: for s = squads[0] the view is inside, so it is copied and the
     extend is right; that extend moves squads[0].units to a bigger buffer and frees the old one. For
     s = squads[1] the view is not inside squads[1]'s buffer, so it is not copied: it is read from the
     freed buffer. Python: [[1, 2, 3, 1, 2], [4, 1, 2]] (probes/py_attack2_output.txt).
     SAVED: the call to reinforce knows, from reinforce's effect summary, that it may grow
     squads[*].units, and that the view comes from squads[0].units, one of them; so it calls the copy
     of reinforce that takes the view as a copy (Python's slice is a copy). Decided when compiling.

   Demo C (regroup): a view kept across clear() through another name of the same squad.
       def regroup(a, b):                        fn regroup(mut a: Squad, b: Squad) -> int:
           first_two = b.units[0:2]                  first_two = b.units[0..2]
           a.units.clear()                           a.units.clear()
           return first_two[0].name                  return first_two[0].name.len()
       regroup(s, s)                             regroup(s, s)
     clear() does not replace the buffer, it drops the elements in it (their names are freed), so
     rule 3's "keep the old buffer alive" keeps nothing that matters. Python: ['ann', 'bob'].
     SAVED: in the alias copy, a view that is still used after a write that may remove, replace or
     move the elements it views is copied just before that write (Python's slice is a copy). */
#include "../../../redesign/bench/harness.h"
#include <string.h>

typedef struct { uint64_t magic, size; } Hdr;
#define LIVE 0x4C4956454C495645ull
#define DEAD 0xDEADDEADDEADDEADull
static int n_freed_read;
static void *xmalloc(size_t n) { Hdr *h = malloc(sizeof(Hdr) + n); h->magic = LIVE; h->size = n; return h + 1; }
static void xfree(void *p) { if (!p) return; Hdr *h = (Hdr *)p - 1; h->magic = DEAD; memset(p, 0xDD, h->size); }
static int dead(const void *p) { return ((const Hdr *)p - 1)->magic != LIVE; }

/* ---- Demo R: lists of ints ---- */
typedef struct { int32_t *data; size_t len, cap; } ListI;
typedef struct { const int32_t *p; size_t n; } SpanI;
typedef struct { ListI units; } Squad;
static void reserve(ListI *l, size_t extra) {
    if (l->len + extra <= l->cap) return;
    size_t c = l->cap ? l->cap : 4; while (c < l->len + extra) c *= 2;
    int32_t *d = xmalloc(c * 4); memcpy(d, l->data, l->len * 4); xfree(l->data); l->data = d; l->cap = c;
}
static void extend_rule4(ListI *l, SpanI s) {             /* the first attack's saved extend */
    int32_t tmp[64];
    if ((uintptr_t)s.p - (uintptr_t)l->data < l->cap * 4) { memcpy(tmp, s.p, s.n * 4); s.p = tmp; }
    reserve(l, s.n);
    memcpy(l->data + l->len, s.p, s.n * 4); l->len += s.n;
}
static void reinforce(Squad *sq, size_t n, SpanI picked) { for (size_t i = 0; i < n; i++) extend_rule4(&sq[i].units, picked); }
static void reinforce_copy(Squad *sq, size_t n, SpanI picked) {   /* SAVED: the copy that owns its view */
    int32_t *own = xmalloc(picked.n * 4); memcpy(own, picked.p, picked.n * 4);
    reinforce(sq, n, (SpanI){own, picked.n});
    xfree(own);
}
static ListI li(const int32_t *v, size_t n) { ListI l = {xmalloc(n * 4), n, n}; memcpy(l.data, v, n * 4); return l; }
static void demo_r(int saved) {
    int32_t a[] = {1, 2, 3}, b[] = {4};
    Squad sq[2] = {{li(a, 3)}, {li(b, 1)}};
    SpanI view = {sq[0].units.data, 2};                    /* squads[0].units[0..2] */
    if (saved) reinforce_copy(sq, 2, view); else reinforce(sq, 2, view);
    printf("  %-8s [", saved ? "saved:" : "written:");
    for (int i = 0; i < 2; i++) {
        printf(i ? ", [" : "[");
        for (size_t k = 0; k < sq[i].units.len; k++) printf(k ? ", %d" : "%d", sq[i].units.data[k]);
        printf("]");
    }
    printf("]\n");
}

/* ---- Demo C: lists of units that own a name ---- */
typedef struct { char *p; size_t n; } Str;
typedef struct { Str name; int hp; } Unit;
typedef struct { Unit *data; size_t len, cap; } ListU;
typedef struct { const Unit *p; size_t n; } SpanU;
typedef struct { ListU units; } SquadU;
static Str str(const char *s) { size_t n = strlen(s); Str r = {xmalloc(n + 1), n}; memcpy(r.p, s, n + 1); return r; }
static Str str_clone(Str s) { Str r = {xmalloc(s.n + 1), s.n}; memcpy(r.p, s.p, s.n + 1); return r; }
static void clear_units(ListU *l) { for (size_t k = 0; k < l->len; k++) xfree(l->data[k].name.p); l->len = 0; }
static const char *name_of(Str s) { if (dead(s.p)) { n_freed_read++; return "<freed>"; } return s.p; }
static void demo_c(int saved) {
    SquadU s = {{xmalloc(3 * sizeof(Unit)), 3, 3}};
    s.units.data[0] = (Unit){str("ann"), 1}; s.units.data[1] = (Unit){str("bob"), 2}; s.units.data[2] = (Unit){str("cy"), 3};
    SquadU *a = &s, *b = &s;                               /* regroup(s, s): the alias copy runs */
    SpanU first_two = {b->units.data, 2};
    Unit own[2];
    if (saved) {                                           /* SAVED: copy the view before the write */
        for (int k = 0; k < 2; k++) own[k] = (Unit){str_clone(first_two.p[k].name), first_two.p[k].hp};
        first_two.p = own;
    }
    clear_units(&a->units);                                /* a.units.clear() */
    printf("  %-8s first_two names: %s, %s", saved ? "saved:" : "written:", name_of(first_two.p[0].name), name_of(first_two.p[1].name));
    printf("   (reads of freed memory: %d)\n", n_freed_read);
    n_freed_read = 0;
}

int main(void) {
    h_start();
    printf("demo R: reinforce(squads, squads[0].units[0..2]); Python [[1, 2, 3, 1, 2], [4, 1, 2]]\n");
    demo_r(0); demo_r(1);
    printf("demo C: regroup(s, s): a view kept across a.units.clear(); Python ['ann', 'bob']\n");
    demo_c(0); demo_c(1);
    return 0;
}
