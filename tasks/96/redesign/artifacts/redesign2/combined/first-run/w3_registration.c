/* combined-w3-registration: the ALTERNATIVE weak mechanism, not the chosen one (see w3.c). W3 (game objects that refer to each other) as the combined memory model's compiler
   would emit it. Report: tasks/96/redesign/phase3b-combined.md, section 6.3.

   The Ember program (the model's syntax):

       open class GameObject:
           id: int
           x: i32
           y: i32
           health: i32
           target: weak GameObject                  # reads as Option[GameObject]; None once it is gone
           fn update(self): pass

       class Ship(GameObject):
           override fn update(self):
               t = self.target
               if t is not None:
                   self.x += sign(t.x - self.x)
                   self.y += sign(t.y - self.y)
       class Rock(GameObject):
           override fn update(self):
               self.x = (self.x + 1) % 1000
       class Turret(GameObject):
           override fn update(self):
               t = self.target
               if t is not None and abs(t.x - self.x) + abs(t.y - self.y) < 300:
                   t.health -= 1

       for frame in range(1000):
           for o in objects:
               o.update()
           for o in objects:                         # the one-pass filter (containers group, A4)
               if o.health <= 0:
                   objects.remove(o)
                   dead += 1
           ...                                       # (c): make one object per dead one

   What the model emits, and nothing else:
   - GameObject is `open` (it has subclasses), so its header keeps the type index beside the count:
     8 bytes, {count u32, type index u32}. The virtual call indexes the method table by the type
     index (plain C's kind table). An object is 40 bytes.
   - Objects come from a slab for their size, with a free list.
   - `weak` fields are cleared by registration (graphs group, A8): reading `target` is a plain load and
     the None test the program writes. Storing into it registers the field's address with the target
     in a side record found through a table keyed by the target's address, and unregisters it from the
     old target. Bit 29 of the count says "has a side record", so an object without one pays nothing
     at death. When an object dies it clears every field registered with it, unregisters its own weak
     field (the holder's death; graphs attack 4.1), and goes back to the slab.
   - Counts: each object is held by `objects` alone (count 1); the filter's removal releases it, and
     it dies at the end of its step. The update loop changes no count: no override can drop a
     GameObject or change `objects` (field effects; the overrides are all in this program).
   - Bounds checks: `objects[rand() % i]` while building (the compiler cannot prove i == len); none on
     `objects[rand() % len(objects)]` (floor modulo by a positive length is below it).
   - Integer overflow checks left out, as in every model. */
#include "../../../redesign/bench/harness.h"   /* path changed when the file moved into first-run/ */
#include <string.h>
#define M 100000

static __declspec(noreturn) void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)

#define HAS_SIDE (1u << 29)
#define COUNT_MASK (HAS_SIDE - 1u)

typedef struct Obj Obj;
struct Obj {
    uint32_t rc, type;            /* header: count (with flag bits), type index (0 Ship, 1 Rock, 2 Turret) */
    int64_t id;
    int32_t x, y, health;
    Obj *target;                  /* weak: cleared when the target dies */
};
_Static_assert(sizeof(Obj) == 40, "Obj layout");

/* ---- the slab for 40-byte objects ---- */
static char *slab_cur, *slab_end;
static Obj *slab_free;
static Obj *obj_alloc(void) {
    if (slab_free) { Obj *o = slab_free; slab_free = *(Obj **)o; return o; }
    if (slab_cur == slab_end) {
        size_t sz = (size_t)4096 * sizeof(Obj);
        slab_cur = malloc(sz); CHECK(slab_cur, "out of memory");
        slab_end = slab_cur + sz;
    }
    Obj *o = (Obj *)slab_cur; slab_cur += sizeof(Obj); return o;
}
static void obj_free(Obj *o) { *(Obj **)o = slab_free; slab_free = o; }

/* ---- weak registration: object address -> the weak fields that point at it ---- */
typedef struct { Obj ***ptr; uint32_t len, cap; } FieldList;
typedef struct { Obj *key; FieldList fl; } Side;
static Side *tab; static size_t tab_cap, tab_len;
static inline size_t hptr(const void *p) {
    uint64_t x = (uint64_t)(uintptr_t)p; x ^= x >> 33; x *= 0xff51afd7ed558ccdull; x ^= x >> 33; return (size_t)x;
}
static void tab_grow(void) {
    size_t oc = tab_cap; Side *ot = tab;
    tab_cap = oc ? oc * 2 : 1024;
    tab = calloc(tab_cap, sizeof(Side)); CHECK(tab, "out of memory");
    for (size_t i = 0; i < oc; i++) if (ot[i].key) {
        size_t j = hptr(ot[i].key) & (tab_cap - 1);
        while (tab[j].key) j = (j + 1) & (tab_cap - 1);
        tab[j] = ot[i];
    }
    free(ot);
}
static Side *side_find(Obj *o) {
    size_t j = hptr(o) & (tab_cap - 1);
    while (tab[j].key != o) { CHECK(tab[j].key, "weak table: record missing"); j = (j + 1) & (tab_cap - 1); }
    return &tab[j];
}
static Side *side_find_or_add(Obj *o) {
    if (o->rc & HAS_SIDE) return side_find(o);
    if ((tab_len + 1) * 2 > tab_cap) tab_grow();
    size_t j = hptr(o) & (tab_cap - 1);
    while (tab[j].key) j = (j + 1) & (tab_cap - 1);
    tab[j].key = o; tab[j].fl = (FieldList){0}; tab_len++;
    o->rc |= HAS_SIDE;
    return &tab[j];
}
static void side_delete(Side *s) {               /* linear probing: shift the following entries back */
    size_t i = (size_t)(s - tab);
    s->key->rc &= ~HAS_SIDE;
    free(s->fl.ptr);
    tab[i].key = NULL; tab_len--;
    for (size_t j = (i + 1) & (tab_cap - 1); tab[j].key; j = (j + 1) & (tab_cap - 1)) {
        size_t home = hptr(tab[j].key) & (tab_cap - 1);
        if (((j - home) & (tab_cap - 1)) >= ((j - i) & (tab_cap - 1))) { tab[i] = tab[j]; tab[j].key = NULL; i = j; }
    }
}
static void weak_unregister(Obj *t, Obj **field) {
    Side *s = side_find(t);
    for (uint32_t k = 0; k < s->fl.len; k++)
        if (s->fl.ptr[k] == field) { s->fl.ptr[k] = s->fl.ptr[--s->fl.len]; break; }
    if (s->fl.len == 0) side_delete(s);
}
static void weak_store(Obj **field, Obj *t) {    /* holder.target = t */
    if (*field) weak_unregister(*field, field);
    if (t) {
        Side *s = side_find_or_add(t);
        if (s->fl.len == s->fl.cap) {
            s->fl.cap = s->fl.cap ? s->fl.cap * 2 : 4;
            s->fl.ptr = realloc(s->fl.ptr, s->fl.cap * sizeof(Obj **)); CHECK(s->fl.ptr, "out of memory");
        }
        s->fl.ptr[s->fl.len++] = field;
    }
    *field = t;
}
static void obj_die(Obj *o) {
    if (o->rc & HAS_SIDE) {                       /* clear every weak field that points here */
        Side *s = side_find(o);
        for (uint32_t k = 0; k < s->fl.len; k++) *s->fl.ptr[k] = NULL;
        side_delete(s);
    }
    if (o->target) weak_unregister(o->target, &o->target);   /* the holder dies: unregister its field */
    obj_free(o);
}
static inline void obj_release(Obj *o) {
    uint32_t c = o->rc - 1;
    o->rc = c;
    if ((c & COUNT_MASK) == 0) obj_die(o);
}

/* ---- the classes' update methods; the type index selects one ---- */
static inline int32_t sign32(int32_t d) { return (d > 0) - (d < 0); }
static void ship_update(Obj *o) {
    Obj *t = o->target;
    if (t) { o->x += sign32(t->x - o->x); o->y += sign32(t->y - o->y); }
}
static void rock_update(Obj *o) { o->x = (o->x + 1) % 1000; }      /* x >= 0: floor modulo == C's % */
static void turret_update(Obj *o) {
    Obj *t = o->target;
    if (t && abs(t->x - o->x) + abs(t->y - o->y) < 300) t->health -= 1;
}
static void (*const UPDATE[3])(Obj *) = {ship_update, rock_update, turret_update};

static Obj *make(int64_t id) {
    int32_t x = (int32_t)(h_rand() % 1000);
    int32_t y = (int32_t)(h_rand() % 1000);
    Obj *o = obj_alloc();
    o->rc = 1; o->type = (uint32_t)(id % 3);
    o->id = id; o->x = x; o->y = y; o->health = (int32_t)(100 + id % 50);
    o->target = NULL;
    return o;
}

typedef struct { Obj **ptr; size_t len, cap; } ObjVec;      /* Array[GameObject] */
static void push(ObjVec *v, Obj *o) {
    if (v->len == v->cap) {
        v->cap = v->cap < 2 ? 4 : v->cap * 2;
        v->ptr = realloc(v->ptr, v->cap * sizeof(Obj *)); CHECK(v->ptr, "out of memory");
    }
    v->ptr[v->len++] = o;
}

int main(void) {
    h_start();
    h_seed(7);
    ObjVec objects = {0};
    for (int64_t i = 0; i < M; i++) {
        Obj *o = make(i);
        if (i > 0) {
            size_t r = h_rand() % (uint64_t)i;
            CHECK(r < objects.len, "index out of bounds");
            weak_store(&o->target, objects.ptr[r]);
        }
        push(&objects, o);                        /* o moves into objects */
    }
    int64_t next_id = M, destroyed = 0;
    for (int f = 0; f < 1000; f++) {
        /* (a) for o in objects: o.update() */
        for (size_t k = 0; k < objects.len; k++) { Obj *o = objects.ptr[k]; UPDATE[o->type](o); }
        /* (b) the one-pass filter: a removed object is released at the end of its step */
        size_t w = 0, before = objects.len;
        for (size_t r = 0; r < before; r++) {
            Obj *o = objects.ptr[r];
            if (o->health <= 0) obj_release(o);
            else objects.ptr[w++] = o;
        }
        objects.len = w;
        int64_t dead = (int64_t)(before - w);
        destroyed += dead;
        /* (c) one new object per dead one */
        for (int64_t d = 0; d < dead; d++) {
            Obj *o = make(next_id++);
            if (objects.len) weak_store(&o->target, objects.ptr[h_rand() % objects.len]);
            push(&objects, o);
        }
    }
    int64_t sum = 0;
    for (size_t k = 0; k < objects.len; k++) sum += (int64_t)objects.ptr[k]->x + objects.ptr[k]->y + objects.ptr[k]->health;
    h_report("combined-w3-registration", (uint64_t)sum + 1000000ull * (uint64_t)destroyed);
    return 0;
}
