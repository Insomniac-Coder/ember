/* combined-w3-registration: the ALTERNATIVE `weak` mechanism, not the chosen one (see w3.c), kept so
   the final measurer can compare the two on W3. Report: tasks/96/redesign/phase3b-combined.md,
   sections 4.3 and 6.3.
   Second run of the combined step; the first run's file (first-run/w3_registration.c) found each
   target's record through a hash table keyed by the target's address. This one finds it through the
   target's count word, the cheapest form the second graphs attack measured ("the record found
   through the object", phase3b-graphs-callbacks-attack.md 4.2: 1.09x and 1.10x of plain C on churn,
   against 1.20x to 1.27x with the table), so the comparison is with registration at its best.

   The Ember program is w3.c's, unchanged.

   What this version emits, and nothing else:
   - GameObject is `open`: an 8-byte header {count word u32, type index u32}; 40-byte objects from a
     slab with a free list; the virtual call indexes the method table by the type index.
   - `weak` fields cleared by registration (the graphs group's answer for A8): reading `target` is a
     plain load and the None test the program writes. Storing into it registers the field's address
     in the target's side record and unregisters it from the old target's. The first registration
     moves the target's count into the record {strong u64, field list} and puts the record's index in
     the count word (SIDE bit); when the last field unregisters from a live object, its count moves
     back. When an object dies it sets every field registered with it to None, frees its record,
     unregisters its own weak field (the holder's death; graphs attack 4.1), and goes back to the
     slab at once.
   - Counts, bounds checks: as w3.c. Integer overflow checks left out, as in every model. */
#include "../../redesign/bench/harness.h"
#include <string.h>
#define M 100000

static __declspec(noreturn) void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)

#define SIDE (1u << 29)        /* the count lives in a side record; the low bits are its index */
#define LOW (SIDE - 1u)

typedef struct Obj Obj;
struct Obj {
    uint32_t rc, type;            /* header: count word, type index (0 Ship, 1 Rock, 2 Turret) */
    int64_t id;
    int32_t x, y, health;
    Obj *target;                  /* weak: set to None when the target dies */
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

/* ---- side records, reached by the index in the count word ---- */
typedef struct { uint64_t strong; Obj ***fields; uint32_t len, cap; } SideRec;
static SideRec *side; static uint32_t side_len, side_cap, side_free_head = UINT32_MAX;
static uint32_t side_alloc(void) {
    if (side_free_head != UINT32_MAX) { uint32_t i = side_free_head; side_free_head = side[i].len; return i; }
    if (side_len == side_cap) {
        side_cap = side_cap ? side_cap * 2 : 1024;
        CHECK(side_cap <= LOW, "too many weakly referenced objects");
        side = realloc(side, side_cap * sizeof(SideRec)); CHECK(side, "out of memory");
    }
    side[side_len].fields = NULL; side[side_len].cap = 0;
    return side_len++;
}
static void side_release(uint32_t i) { side[i].len = side_free_head; side_free_head = i; }   /* keeps its field buffer */

static void weak_unregister(Obj *t, Obj **field) {
    uint32_t i = t->rc & LOW;
    SideRec *s = &side[i];
    for (uint32_t k = 0; k < s->len; k++)
        if (s->fields[k] == field) { s->fields[k] = s->fields[--s->len]; break; }
    if (s->len == 0 && s->strong <= LOW) {        /* no field left: the count goes back to the header */
        t->rc = (t->rc & ~(LOW | SIDE)) | (uint32_t)s->strong;
        side_release(i);
    }
}
static void weak_store(Obj **field, Obj *t) {     /* holder.target = t */
    if (*field) weak_unregister(*field, field);
    if (t) {
        if (!(t->rc & SIDE)) {
            uint32_t i = side_alloc();
            side[i].strong = t->rc & LOW; side[i].len = 0;
            t->rc = (t->rc & ~LOW) | SIDE | i;
        }
        SideRec *s = &side[t->rc & LOW];
        if (s->len == s->cap) {
            s->cap = s->cap ? s->cap * 2 : 4;
            s->fields = realloc(s->fields, s->cap * sizeof(Obj **)); CHECK(s->fields, "out of memory");
        }
        s->fields[s->len++] = field;
    }
    *field = t;
}
static void obj_die(Obj *o) {
    if (o->rc & SIDE) {                           /* set every weak field that points here to None */
        uint32_t i = o->rc & LOW;
        for (uint32_t k = 0; k < side[i].len; k++) *side[i].fields[k] = NULL;
        side_release(i);
    }
    if (o->target) weak_unregister(o->target, &o->target);   /* the holder dies: unregister its field */
    obj_free(o);
}
static inline void obj_release(Obj *o) {
    if (!(o->rc & SIDE)) { if ((--o->rc & LOW) == 0) obj_die(o); return; }
    if (--side[o->rc & LOW].strong == 0) obj_die(o);
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
        for (size_t k = 0; k < objects.len; k++) { Obj *o = objects.ptr[k]; UPDATE[o->type](o); }
        size_t w = 0, before = objects.len;
        for (size_t r = 0; r < before; r++) {
            Obj *o = objects.ptr[r];
            if (o->health <= 0) obj_release(o);
            else objects.ptr[w++] = o;
        }
        objects.len = w;
        int64_t dead = (int64_t)(before - w);
        destroyed += dead;
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
