/* A4 memory attacks on the one-pass filter loop (phase3b-containers.md section 4.3, rules 7, 9, 10)
   and their fixes. Each demo is the C the rules compile an Ember loop into, over a list of elements
   that own memory (Unit has a String name). A poisoning allocator makes the damage visible: a freed
   block is filled with 0xDD and kept (never handed back), so a read of freed memory and a second free
   of one block are both counted instead of being silent.
   Each demo runs twice: "as written" (the report's model: ember_swapfill.c, the walkers count in the
   top byte of the capacity word, the re-checks r < len and w <= r) and "saved" (the fix). A panic
   (Ember aborts, [PAN-1]) ends the demo; it is the correct outcome where Python would also fail. */
#include "../harness.h"
#include <string.h>
#include <setjmp.h>

typedef struct { uint64_t magic, size; } Hdr;
#define LIVE 0x4C4956454C495645ull
#define DEAD 0xDEADDEADDEADDEADull
static int n_double_free, n_freed_read, n_two_owners;
static void *xmalloc(size_t n) { Hdr *h = malloc(sizeof(Hdr) + n); h->magic = LIVE; h->size = n; return h + 1; }
static void xfree(void *p) {
    if (!p) return;
    Hdr *h = (Hdr *)p - 1;
    if (h->magic != LIVE) { n_double_free++; return; }
    h->magic = DEAD; memset(p, 0xDD, h->size);          /* quarantined, never handed back */
}
static void *xrealloc(void *p, size_t n) {
    void *q = xmalloc(n);
    if (p) { size_t old = ((Hdr *)p - 1)->size; memcpy(q, p, old < n ? old : n); xfree(p); }
    return q;
}
static jmp_buf on_panic;
static void panic(const char *m) { printf("    panic: %s\n", m); longjmp(on_panic, 1); }

typedef struct { char *p; size_t n; } Str;
static Str str(const char *s) { size_t n = strlen(s); Str r = {xmalloc(n + 1), n}; memcpy(r.p, s, n + 1); return r; }
static void str_read(Str s) { if (((Hdr *)s.p - 1)->magic != LIVE) n_freed_read++; }   /* print(s) */

typedef struct { Str name; int hp; } Unit;
typedef struct { Unit *data; size_t len, cap; } List;   /* cap's top byte: the walkers count (rule 9) */
#define ONE (1ull << 56)
#define CAP(l) ((l)->cap & (ONE - 1))
#define WALKERS(l) ((l)->cap >> 56)
static void push(List *l, Unit u) {
    if (l->len == CAP(l)) { size_t c = CAP(l) ? CAP(l) * 2 : 4; l->data = xrealloc(l->data, c * sizeof(Unit)); l->cap = (l->cap & ~(ONE - 1)) | c; }
    l->data[l->len++] = u;
}
static void remove_at(List *l, size_t k) {               /* rule 9: panics while the list is walked */
    if (WALKERS(l)) panic("list changed while a loop over it was running");
    if (k >= l->len) panic("index out of bounds");
    xfree(l->data[k].name.p);
    memmove(&l->data[k], &l->data[k + 1], (l->len - k - 1) * sizeof(Unit));
    l->len--;
}
static void drop_list(List *l) {
    for (size_t k = 0; k < l->len; k++) xfree(l->data[k].name.p);
    xfree(l->data); l->data = NULL; l->len = l->cap = 0;
}
static List make(const int *hp, int n) {
    List l = {0}; char nm[16];
    for (int k = 0; k < n; k++) { snprintf(nm, sizeof nm, "u%d", k); push(&l, (Unit){str(nm), hp[k]}); }
    return l;
}
static void count_two_owners(const List *a, const List *b) {
    for (size_t i = 0; i < a->len; i++) for (size_t j = 0; j < b->len; j++)
        if (a->data[i].name.p == b->data[j].name.p) n_two_owners++;
    for (size_t i = 0; i < a->len; i++) for (size_t j = i + 1; j < a->len; j++)
        if (a->data[i].name.p == a->data[j].name.p) n_two_owners++;
}
static void report(const char *demo, int saved) {
    printf("  %-8s double frees %d, reads of freed memory %d, elements owned twice %d\n",
           saved ? "saved:" : "written:", n_double_free, n_freed_read, n_two_owners);
    n_double_free = n_freed_read = n_two_owners = 0; (void)demo;
}

/* Demo 1. Rule 10's swap-fill mode keeps the element in registers (ember_swapfill.c); a removed element
   is "not stored", so its slot keeps the bits from before the body ran.
       for u in units:
           u.name = "renamed"          # the old String is dropped
           if u.hp <= 0: units.remove(u)
           on_step()                   # code the compiler cannot see: this is what selects the mode */
static void demo1(int saved) {
    int hp[] = {5, 0, 5, 0}; List l = make(hp, 4);
    if (!setjmp(on_panic)) {
        l.cap += ONE;
        size_t r = 0, w = 0;
        while (r < l.len) {
            Unit e = l.data[r];
            xfree(e.name.p); e.name = str("renamed");
            int removed = e.hp <= 0;
            if (!removed) { if (w != r) l.data[r] = l.data[w]; l.data[w] = e; w++; }
            else if (saved) l.data[r] = e;               /* FIX: the removed element goes back to its slot */
            r++;
        }
        for (size_t k = w; k < l.len; k++) xfree(l.data[k].name.p);   /* rule 7: dropped at the loop's end */
        l.len = w; l.cap -= ONE;
        drop_list(&l);
    }
    report("1", saved);
}

/* Demo 2. A callback renames every unit while the loop holds the current one in registers.
       for u in w.units:
           u.hp -= 1
           w.on_step()                 # unseen: for x in self.units: x.name = x.name.upper()
           print(u.name)
   Writing an element is allowed while the list is walked (rule 9 forbids only removing, inserting,
   reordering, replacing and dropping). */
static void on_step_rename(List *l) { for (size_t k = 0; k < l->len; k++) { xfree(l->data[k].name.p); l->data[k].name = str("UPPER"); } }
static void demo2(int saved) {
    int hp[] = {5, 5, 5}; List l = make(hp, 3);
    if (!setjmp(on_panic)) {
        l.cap += ONE;
        size_t r = 0, w = 0;
        while (r < l.len) {
            Unit e = l.data[r];
            e.hp -= 1;
            if (saved) l.data[r] = e;                    /* FIX: spill before code the compiler cannot see */
            on_step_rename(&l);
            if (!(r < l.len && w <= r)) panic("list changed under the loop");   /* rule 10 */
            if (saved) e = l.data[r];                    /* FIX: reload after it */
            str_read(e.name);
            if (w != r) l.data[r] = l.data[w];
            l.data[w] = e; w++; r++;
        }
        l.len = w; l.cap -= ONE;
        drop_list(&l);
    }
    report("2", saved);
}

/* Demo 3. The walkers count is 8 bits. 255 loops (or names, rule 5 of A2) over this list are running
   in the frames above, e.g. a recursive event dispatcher; this loop is the 256th and the count wraps
   to 0. On the third step a callback removes element 0, which the (wrapped) count now allows.
       for u in units:
           if u.hp <= 0: units.remove(u)
           on_step()                   # unseen: on one step, self.units.remove(0)  */
static void demo3(int saved) {
    int hp[] = {5, 0, 5, 5, 5}; List l = make(hp, 5);
    if (!setjmp(on_panic)) {
        l.cap += 255 * ONE;                              /* the 255 outer walkers */
        if (saved && WALKERS(&l) == 255) panic("more than 255 loops over one list at once");  /* FIX */
        l.cap += ONE;                                    /* this loop: 255 + 1 wraps to 0 */
        size_t r = 0, w = 0;
        while (r < l.len) {
            Unit e = l.data[r];
            int removed = e.hp <= 0;
            if (r == 2) remove_at(&l, 0);                /* the callback */
            if (!(r < l.len && w <= r)) panic("list changed under the loop");   /* rule 10: both hold here */
            if (!removed) { if (w != r) l.data[r] = l.data[w]; l.data[w] = e; w++; }
            r++;
        }
        for (size_t k = w; k < l.len; k++) xfree(l.data[k].name.p);
        l.len = w; l.cap -= ONE;
        count_two_owners(&l, &(List){0});
        l.cap = CAP(&l);
        drop_list(&l);
    }
    report("3", saved);
}

/* Demo 4. A callback moves the walked list out of its field (mem.swap with another field). Moving is
   not an operation on the list, so the count in the list's own header does not see it; the loop goes
   on over the other list, at the same place, with its old r and w.
       for u in w.units:
           w.on_step()                 # unseen: mem.swap(self.units, self.reserve)
   Saved: the walking state is kept in the field's access word ([EXC-19]); mem.swap(self.units, ...)
   passes the field to a `mut` parameter, a write access, which panics while the field is walked. */
typedef struct { List units, reserve; uint32_t units_word; } World;   /* units_word: [EXC-19]'s access word */
#define WALKING 0x40000000u
static void demo4(int saved) {
    int hp[] = {5, 5, 5}, hq[] = {7, 7, 7, 7};
    World wd = {make(hp, 3), make(hq, 4), 0};
    if (!setjmp(on_panic)) {
        List *l = &wd.units;                             /* the loop reaches the list through its field */
        if (saved) wd.units_word |= WALKING; else l->cap += ONE;
        size_t r = 0, w = 0;
        while (r < l->len) {
            Unit e = l->data[r];
            if (r == 1) {                                /* the callback: mem.swap(self.units, self.reserve) */
                if (saved && wd.units_word) panic("write access to World.units while a loop walks it");
                List t = wd.units; wd.units = wd.reserve; wd.reserve = t;
            }
            if (!(r < l->len && w <= r)) panic("list changed under the loop");   /* rule 10 */
            if (w != r) l->data[r] = l->data[w];
            l->data[w] = e; w++; r++;
        }
        l->len = w; if (saved) wd.units_word &= ~WALKING; else l->cap -= ONE;
        count_two_owners(&wd.units, &wd.reserve);
        printf("    after the loop: walkers byte of the list now in units %llu, of the list now in reserve %llu\n",
               (unsigned long long)WALKERS(&wd.units), (unsigned long long)WALKERS(&wd.reserve));
        wd.units.cap = CAP(&wd.units); wd.reserve.cap = CAP(&wd.reserve);
        drop_list(&wd.units); drop_list(&wd.reserve);
    }
    report("4", saved);
}

int main(void) {
    h_start();
    printf("demo 1: a removed element's slot keeps its pre-update bits (swap-fill mode)\n"); demo1(0); demo1(1);
    printf("demo 2: a callback rewrites the current element held in registers\n");          demo2(0); demo2(1);
    printf("demo 3: the 8-bit walkers count wraps at 256\n");                               demo3(0); demo3(1);
    printf("demo 4: a callback moves the walked list out of its field\n");                   demo4(0); demo4(1);
    return 0;
}
