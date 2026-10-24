/* weakarr.c: an observer list as a `weak` list (A8's `weak`, cleared by registration), under churn.
   The Ember program (proposed syntax):

       class Subject:
           observers: Array[weak Observer] = []
       # 1,000 observers subscribed; then 300,000 operations, each at a random position k:
       #   99 in 100: the observer at k unsubscribes and subscribes again (moves to the end):
       #       o = s.observers.remove(k); s.observers.push(o)
       #   1 in 100: the observer at k dies (its weak slot becomes None), the program removes the
       #       None at k, and a new observer subscribes: s.observers.push(Observer(...))

     -DPLAIN   plain C: a list of observer pointers; a dying observer unsubscribes itself (search,
               memory move), as C observer code does.
     -DPLAIN_TRACKED  plain C as written when an observer may be in several lists: each observer
               keeps the list of subjects it is in (a pointer field), updated on subscribe and
               unsubscribe, and unsubscribes from each when it dies (what -DARRAY does, without the
               side-table lookup, since a C object can have the field).
     -DSLOTS   the report's registration, made safe: each slot's address is registered with its
               target in the side table (keyed by the object's address, as w2_zweak.c); the list
               re-registers every element whose slot moves: on growth (the report's words) and on
               every shift by a removal (needed, attacks2.exe 9). A death writes None into each
               registered slot.
     -DARRAY -DCACHE1  the same, with a one-entry cache of the last side record found (an object
               taken out of a list is often put back at once): the second lookup is a compare.
     -DARRAY   the save: the list registers itself (the address of its {ptr, len, cap} inside the
               holder object, which never moves) with each target; nothing changes when slots move;
               a death scans each registered list for the dying object.
     -DBUFFER  the save, made robust (attacks2.exe 13): the list's buffer carries a one-word prefix
               (its live length, kept by the list's own push and remove) and the buffer's address is
               what is registered, so moving the list value itself (a move out of the field) changes
               nothing; a growth that moves the buffer re-registers each element once (amortised: one
               lookup per push); a death scans each registered buffer. With -DCACHE1 as above.
     -DBUFFER -DDIRECT  the same, with the side record found through a pointer in the object instead
               of the table (Swift keeps a weakly referenced object's side-table pointer in its
               reference-count word; here an extra field stands for that word): what is left when
               the lookup is gone.
   Every program does the same operations in the same order (one random source) and prints the same
   checksum. Overflow checks are left out, as in every model. */
#include "ember_model.h"
#define NOBS 1000
#define OPS 300000

#ifdef PLAIN_TRACKED
typedef struct Obs { uint32_t rc, type; int64_t id; EVec subjects; } Obs;
#elif defined(DIRECT)
typedef struct Obs { uint32_t rc, type; int64_t id; struct Side *side; } Obs;
#else
typedef struct Obs { uint32_t rc, type; int64_t id; } Obs;
#endif
typedef struct { Obs **ptr; size_t len, cap; } OVec;
typedef struct { uint32_t rc, type; OVec observers; } Subject;

static Obs *obs_new(int64_t id) {
    Obs *o = malloc(sizeof *o); o->rc = 1; o->type = 1; o->id = id;
#ifdef DIRECT
    o->side = NULL;
#endif
#ifdef PLAIN_TRACKED
    o->subjects = (EVec){0};
#endif
    return o;
}

#if defined(SLOTS) || defined(ARRAY) || defined(BUFFER)
/* the side table: object address -> its registrations (open addressing, as w2_zweak.c) */
typedef struct Side { Obs *key; EVec regs; } Side;
static Side *tab; static size_t tab_cap, tab_len;
static inline size_t hash_ptr(const void *p) { uint64_t x = (uint64_t)(uintptr_t)p; x ^= x >> 33; x *= 0xff51afd7ed558ccdull; x ^= x >> 33; return (size_t)x; }
#ifdef CACHE1
static Obs *g_last_key; static Side *g_last_side;
#endif
static Side *side_find(Obs *o, int create) {
#ifdef DIRECT
    if (!o->side && create) { o->side = calloc(1, sizeof(Side)); o->side->key = o; }
    return o->side;
#endif
#ifdef CACHE1
    if (o == g_last_key) return g_last_side;
#endif
    if (create && (tab_len + 1) * 2 > tab_cap) {
#ifdef CACHE1
        g_last_key = NULL;                             /* the table moves: forget the cached record */
#endif
        size_t oc = tab_cap; Side *ot = tab;
        tab_cap = oc ? oc * 2 : 1024; tab = calloc(tab_cap, sizeof(Side)); tab_len = 0;
        for (size_t i = 0; i < oc; i++) if (ot[i].key) {
            size_t j = hash_ptr(ot[i].key) & (tab_cap - 1);
            while (tab[j].key) j = (j + 1) & (tab_cap - 1);
            tab[j] = ot[i]; tab_len++;
        }
        free(ot);
    }
    size_t j = hash_ptr(o) & (tab_cap - 1);
    while (tab[j].key) {
        if (tab[j].key == o) {
#ifdef CACHE1
            g_last_key = o; g_last_side = &tab[j];
#endif
            return &tab[j];
        }
        j = (j + 1) & (tab_cap - 1);
    }
    if (!create) return NULL;
    tab[j].key = o; tab[j].regs = (EVec){0}; tab_len++;
#ifdef CACHE1
    g_last_key = o; g_last_side = &tab[j];
#endif
    return &tab[j];
}
static inline void reg_add(Obs *o, void *addr) { evec_push(&side_find(o, 1)->regs, addr); }
static inline void reg_remove(Obs *o, void *addr) {
    Side *s = side_find(o, 0);
    evec_remove_at(&s->regs, evec_index_of(&s->regs, addr));
}
#endif
#if defined(SLOTS) || defined(BUFFER)
static inline void reg_move(Obs *o, void *from, void *to) {
    Side *s = side_find(o, 0);
    s->regs.ptr[evec_index_of(&s->regs, from)] = to;
}
#endif

#ifdef BUFFER
typedef struct { size_t len; } BufHdr;                 /* the buffer's prefix */
#define BUF_OF(a) ((BufHdr *)((char *)(a)->ptr - sizeof(BufHdr)))
#endif
static void ov_push(OVec *a, Obs *o) {
#ifdef BUFFER
    if (a->len == a->cap) {
        size_t cap = a->cap < 2 ? 4 : a->cap * 2;
        BufHdr *nh = malloc(sizeof(BufHdr) + cap * sizeof(Obs *));
        Obs **nslots = (Obs **)(nh + 1);
        BufHdr *oh = a->ptr ? BUF_OF(a) : NULL;
        if (a->len) memcpy(nslots, a->ptr, a->len * sizeof(Obs *));
        for (size_t i = 0; i < a->len; i++) if (nslots[i]) reg_move(nslots[i], oh, nh);   /* the buffer moved */
        free(oh);
        a->ptr = nslots; a->cap = cap; nh->len = a->len;
    }
    a->ptr[a->len] = o;
    reg_add(o, BUF_OF(a));
    a->len++;
    BUF_OF(a)->len = a->len;
    return;
#endif
    if (a->len == a->cap) {
        size_t cap = a->cap < 2 ? 4 : a->cap * 2;
        Obs **old = a->ptr;
        a->ptr = realloc(a->ptr, cap * sizeof(Obs *));
        a->cap = cap;
#ifdef SLOTS
        if (old && a->ptr != old)                      /* the buffer moved: every slot re-registers */
            for (size_t i = 0; i < a->len; i++)
                if (a->ptr[i]) reg_move(a->ptr[i], (void *)&old[i], (void *)&a->ptr[i]);
#else
        (void)old;
#endif
    }
    a->ptr[a->len] = o;
#if defined(SLOTS)
    reg_add(o, &a->ptr[a->len]);
#elif defined(ARRAY)
    reg_add(o, a);
#elif defined(PLAIN_TRACKED)
    evec_push(&o->subjects, a);
#endif
    a->len++;
}
static Obs *ov_remove(OVec *a, size_t k) {
    e_bounds(k, a->len);
    Obs *o = a->ptr[k];
#ifdef BUFFER
    if (o) reg_remove(o, BUF_OF(a));
    memmove(&a->ptr[k], &a->ptr[k + 1], (a->len - k - 1) * sizeof(Obs *));
    a->len--;
    BUF_OF(a)->len = a->len;
    return o;
#endif
#if defined(SLOTS)
    if (o) reg_remove(o, &a->ptr[k]);
#elif defined(ARRAY)
    if (o) reg_remove(o, a);
#elif defined(PLAIN_TRACKED)
    evec_remove_at(&o->subjects, evec_index_of(&o->subjects, a));
#endif
    memmove(&a->ptr[k], &a->ptr[k + 1], (a->len - k - 1) * sizeof(Obs *));
#ifdef SLOTS
    for (size_t i = k; i + 1 < a->len; i++)            /* every shifted slot re-registers */
        if (a->ptr[i]) reg_move(a->ptr[i], &a->ptr[i + 1], &a->ptr[i]);
#endif
    a->len--;
    return o;
}
static void obs_die(Subject *s, Obs *o) {
#if defined(BUFFER)
    Side *r = side_find(o, 0);
    for (size_t i = 0; i < r->regs.len; i++) {
        BufHdr *h = r->regs.ptr[i];
        Obs **slots = (Obs **)(h + 1);
        for (size_t k = 0; k < h->len; k++) if (slots[k] == o) slots[k] = NULL;
    }
    r->regs.len = 0;
    (void)s;
#elif defined(SLOTS)
    Side *r = side_find(o, 0);
    for (size_t i = 0; i < r->regs.len; i++) *(Obs **)r->regs.ptr[i] = NULL;
    r->regs.len = 0;
    (void)s;
#elif defined(ARRAY)
    Side *r = side_find(o, 0);
    for (size_t i = 0; i < r->regs.len; i++) {
        OVec *a = r->regs.ptr[i];
        for (size_t k = 0; k < a->len; k++) if (a->ptr[k] == o) a->ptr[k] = NULL;
    }
    r->regs.len = 0;
    (void)s;
#elif defined(PLAIN_TRACKED)
    for (size_t i = 0; i < o->subjects.len; i++) {   /* unsubscribe from every list it is in */
        OVec *a = o->subjects.ptr[i];
        size_t k = 0;
        while (a->ptr[k] != o) k++;
        memmove(&a->ptr[k], &a->ptr[k + 1], (a->len - k - 1) * sizeof(Obs *));
        a->len--;
    }
    free(o->subjects.ptr);
    (void)s;
#else
    /* plain C: the observer unsubscribes itself */
    OVec *a = &s->observers;
    size_t k = 0;
    while (a->ptr[k] != o) k++;
    memmove(&a->ptr[k], &a->ptr[k + 1], (a->len - k - 1) * sizeof(Obs *));
    a->len--;
#endif
    free(o);
}

int main(void) {
    h_start();
    h_seed(7);
    Subject *s = calloc(1, sizeof *s); s->rc = 1; s->type = 2;
    for (int64_t i = 0; i < NOBS; i++) ov_push(&s->observers, obs_new(i));
    for (int64_t op = 0; op < OPS; op++) {
        size_t k = h_rand() % s->observers.len;
        if (op % 100 == 0) {
            obs_die(s, s->observers.ptr[k]);
#if defined(SLOTS) || defined(ARRAY) || defined(BUFFER)
            ov_remove(&s->observers, k);               /* the program removes the None */
#endif
            ov_push(&s->observers, obs_new(NOBS + op));
        } else {
            Obs *o = ov_remove(&s->observers, k);
            ov_push(&s->observers, o);
        }
    }
    uint64_t sum = 0;
    for (size_t k = 0; k < s->observers.len; k++) sum += (uint64_t)(s->observers.ptr[k]->id * (int64_t)(k + 1));
#if defined(SLOTS)
    h_report("weakarr-slots", sum);
#elif defined(BUFFER) && defined(DIRECT)
    h_report("weakarr-buffer-direct", sum);
#elif defined(BUFFER) && defined(CACHE1)
    h_report("weakarr-buffer-cache1", sum);
#elif defined(BUFFER)
    h_report("weakarr-buffer", sum);
#elif defined(ARRAY) && defined(CACHE1)
    h_report("weakarr-array-cache1", sum);
#elif defined(ARRAY)
    h_report("weakarr-array", sum);
#elif defined(PLAIN_TRACKED)
    h_report("weakarr-plain-tracked", sum);
#else
    h_report("weakarr-plain", sum);
#endif
    return 0;
}
