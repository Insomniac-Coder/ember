/* A4 memory attack on the first attack's saved mode for code the compiler cannot see
   (phase3b-containers-attack.md 5.5 and section 9, A4 rule 5): "for a list reached through a class
   field, the loop holds a walking access in the field's access word ... It allows reads, writes of
   elements and appends; assigning, moving, dropping or reordering the list ... panic."
   The walked list here is NOT a class field: it is a field of a struct element of one.
       class World:                                   class World:
           squads: Array[Squad]   (Squad: units)          squads: Array[Squad]
       for u in self.squads[0].units:                 for u in self.squads[0].units:
           u.hp -= 1                                       u.hp -= 1
           if u.hp <= 0: self.squads[0].units.remove(u)    if u.hp <= 0: self.squads[0].units.remove(u)
           self.on_step(self)                              self.on_step(self)     # code the compiler cannot see
   The only class field on the path is `squads`, so that is the access word the walk sets. Two
   callbacks that the walking access allows:
     (i)  self.squads[0] = Squad(units=[Unit(7)])   a write of an element of `squads`
     (ii) self.squads.push(Squad(units=[]))         an append to `squads`
   (i) drops the walked list (its removed elements were already dropped at the end of their steps:
   double frees); (ii) moves the Squad that holds the walked list's header, and the loop, which keeps
   the header's address as the first attack's model does for a class field, reads it from freed memory.
   Python (probes/py_attack2_output.txt, check 7): the loop goes on over the old list object.
   SAVED: the walk publishes its PATH (field `squads`, index 0), not only the field; while it runs,
   replacing, removing, inserting before, reordering or dropping any value on the path panics, appending
   to an outer list is allowed, and the loop finds its list again through the path after each call the
   compiler cannot see (it never keeps the header's address across one).
   Poisoning allocator: a freed block is filled with 0xDD and kept, so double frees and reads of freed
   memory are counted, and nothing crashes. A panic ends the demo, as Ember's abort would. */
#include "../../../redesign/bench/harness.h"
#include <string.h>
#include <setjmp.h>

typedef struct { uint64_t magic, size; } Hdr;
#define LIVE 0x4C4956454C495645ull
#define DEAD 0xDEADDEADDEADDEADull
static int n_double_free, n_freed_read;
static void *xmalloc(size_t n) { Hdr *h = malloc(sizeof(Hdr) + n); h->magic = LIVE; h->size = n; return h + 1; }
static void xfree(void *p) {
    if (!p) return;
    Hdr *h = (Hdr *)p - 1;
    if (h->magic != LIVE) { n_double_free++; return; }
    h->magic = DEAD; memset(p, 0xDD, h->size);
}
static int dead(const void *p) { return ((const Hdr *)p - 1)->magic != LIVE; }
static jmp_buf on_panic;
static void panic(const char *m) { printf("    panic: %s\n", m); longjmp(on_panic, 1); }

typedef struct { char *p; } Str;
static Str str(const char *s) { size_t n = strlen(s); Str r = {xmalloc(n + 1)}; memcpy(r.p, s, n + 1); return r; }
typedef struct { Str name; int hp; } Unit;
typedef struct { Unit *data; size_t len, cap; } ListU;
typedef struct { ListU units; } Squad;
typedef struct { Squad *data; size_t len, cap; } ListS;
typedef struct { size_t r, w; int cur_removed; size_t path_index; } Walker;
#define WALKING 0x40000000u
typedef struct { ListS squads; uint32_t squads_word; Walker *walker; } World;

static ListU units(const int *hp, int n, const char *tag) {
    ListU l = {xmalloc(n * sizeof(Unit)), (size_t)n, (size_t)n}; char nm[16];
    for (int k = 0; k < n; k++) { snprintf(nm, sizeof nm, "%s%d", tag, k); l.data[k] = (Unit){str(nm), hp[k]}; }
    return l;
}
static void drop_units(ListU *l) { for (size_t k = 0; k < l->len; k++) xfree(l->data[k].name.p); xfree(l->data); }

/* the two callbacks, as the library compiles squads[0] = ... and squads.push(...) */
static int saved_mode;
static void squads_set(World *w, size_t k, Squad v) {
    if (saved_mode && (w->squads_word & WALKING) && k == w->walker->path_index)
        panic("squads[0] holds a list a loop is walking");          /* SAVED: a value on the path */
    drop_units(&w->squads.data[k].units);                            /* the old Squad is dropped */
    w->squads.data[k] = v;
}
static void squads_push(World *w, Squad v) {                         /* appends: allowed in both */
    if (w->squads.len == w->squads.cap) {
        size_t c = w->squads.cap * 2;
        Squad *d = xmalloc(c * sizeof(Squad)); memcpy(d, w->squads.data, w->squads.len * sizeof(Squad));
        xfree(w->squads.data); w->squads.data = d; w->squads.cap = c;
    }
    w->squads.data[w->squads.len++] = v;
}
static int which_cb;
static void on_step(World *w) {
    if (which_cb == 1) { int hp7[] = {7}; squads_set(w, 0, (Squad){units(hp7, 1, "n")}); }
    else { squads_push(w, (Squad){units(NULL, 0, "x")}); }
}

static void demo(int cb, int saved) {
    which_cb = cb; saved_mode = saved;
    int hp[] = {1, 0, 3};
    World wd = {{xmalloc(sizeof(Squad)), 1, 1}, 0, NULL};
    wd.squads.data[0] = (Squad){units(hp, 3, "u")};
    if (!setjmp(on_panic)) {
        Walker wk = {0, 0, 0, 0};
        wd.squads_word |= WALKING; wd.walker = &wk;
        ListU *l = &wd.squads.data[0].units;                  /* the walked list's header */
        Squad *base = wd.squads.data;                         /* the buffer that header lives in */
        size_t r = 0, w = 0;
        while (r < l->len) {
            Unit e = l->data[r];
            e.hp -= 1;
            int removed = e.hp <= 0;                          /* self.squads[0].units.remove(u) */
            l->data[r] = e;                                   /* spill e */
            wk.r = r; wk.w = w; wk.cur_removed = removed;
            if (r == 2) on_step(&wd);                         /* the call the compiler cannot see */
            if (saved) { l = &wd.squads.data[wk.path_index].units; base = wd.squads.data; }   /* SAVED: by its path */
            if (dead(base)) { n_freed_read++; printf("    the loop read its list's header from freed memory\n"); break; }
            r = wk.r; w = wk.w;
            if (!(r < l->len && w <= r)) panic("list changed under the loop");
            e = l->data[r];                                   /* reload */
            if (wk.cur_removed) xfree(e.name.p);              /* a removed element is dropped at the end of its step */
            else { l->data[w] = e; w++; }
            r++;
        }
        if (!dead(base)) l->len = w;
        wd.squads_word &= ~WALKING; wd.walker = NULL;
        printf("    after the loop: squads[0].units has %zu units", wd.squads.data[0].units.len);
        for (size_t k = 0; k < wd.squads.data[0].units.len; k++) printf(k ? ", hp %d" : ": hp %d", wd.squads.data[0].units.data[k].hp);
        printf("\n");
    }
    printf("  %-8s double frees %d, reads of freed memory %d\n", saved ? "saved:" : "written:", n_double_free, n_freed_read);
    n_double_free = n_freed_read = 0;
}

int main(void) {
    h_start();
    printf("demo (i): the callback replaces squads[0], which holds the walked list\n"); demo(1, 0); demo(1, 1);
    printf("demo (ii): the callback appends to squads, moving the walked list's header\n"); demo(2, 0); demo(2, 1);
    return 0;
}
