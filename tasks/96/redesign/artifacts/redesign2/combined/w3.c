/* combined-w3: W3 (game objects that refer to each other) as the combined memory model's compiler
   would emit it. Report: tasks/96/redesign/phase3b-combined.md, section 6.3.
   (w3_registration.c is the same program with the other `weak` mechanism, kept for the measurer.)
   Second run of the combined step; the first run's file is first-run/w3.c. What changed: the side
   record's two counts are 64 bits (phase3b-ownership-attack.md 5.5: a count that leaves the header
   because it is large must not overflow at 2^32 in the record).

   The Ember program (the model's syntax):

       open class GameObject:
           id: int
           x: i32
           y: i32
           health: i32
           target: weak GameObject = None           # reads as Option[GameObject]; None once it is gone
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
               if t is not None and (t.x - self.x).abs() + (t.y - self.y).abs() < 300:
                   t.health -= 1

       for frame in 0..1000:
           for o in objects:
               o.update()
           for o in objects:                         # the one-pass filter (containers group, A4)
               if o.health <= 0:
                   objects.remove(o)
                   dead += 1
           ...                                       # (c): one new object per dead one

   What the model emits, and nothing else:
   - GameObject is `open` (it has subclasses), so its header keeps the type index beside the count
     word: 8 bytes, {count word u32, type index u32}. The virtual call indexes the method table by the
     type index (plain C's kind table). An object is 40 bytes.
   - Objects come from a slab for their size, with a free list.
   - `weak`, tested on read: a weak field is a plain pointer. Reading it tests one bit of the
     target's count word (DEAD) and gives None when it is set; the read writes nothing. The first
     weak store to an object moves its count into a side record {strong u64, weak u64} and puts the
     record's index in the count word (SIDE bit), as Swift moves its counts to a side table; storing
     into a weak field adds one to the target's weak count and takes one from the old target's. When
     the strong count reaches zero the object's fields are dropped and DEAD is set; its 40-byte block
     stays until the last weak field that points at it is overwritten or dies (as Rust's Rc/Weak
     allocation does), then goes back to the slab. When the weak count of a live object reaches zero,
     the count moves back to the header.
     Count word bits: 31 shared and 30 claim (threads group; unused here), 29 SIDE, 28 DEAD, 0..27
     the count or the side record's index.
   - Counts: each object is held by `objects` alone (count 1); the filter's removal releases it at the
     end of its step. The update loop changes no count: no override can drop a GameObject or change
     `objects` (effect summaries; the overrides are all in this program). `t` is used inside its own
     statement with nothing in between that may drop a GameObject, so the weak read takes no count.
   - Bounds checks: `objects[rand() % i]` while building (the compiler cannot prove i == len); none on
     `objects[rand() % len(objects)]` (floor modulo by a positive length is below it, [TYP-28]).
   - Integer overflow checks left out, as in every model.
   Self-check build: -DPOISON never reuses a block, marks freed ones, and checks every weak read. */
#include "../../redesign/bench/harness.h"
#include <string.h>
#define M 100000

static __declspec(noreturn) void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)

#define SIDE (1u << 29)        /* the count lives in a side record; the low bits are its index */
#define DEAD (1u << 28)        /* the fields are dropped; only weak fields keep the block */
#define LOW (DEAD - 1u)        /* the count, or the side record's index */

typedef struct Obj Obj;
struct Obj {
    uint32_t rc, type;          /* header: count word, type index (0 Ship, 1 Rock, 2 Turret) */
    int64_t id;
    int32_t x, y, health;
    Obj *target;                /* weak */
};
_Static_assert(sizeof(Obj) == 40, "Obj layout");

/* ---- the slab for 40-byte objects ---- */
static char *slab_cur, *slab_end;
static Obj *slab_free;
#ifdef POISON   /* self-check build: blocks are never reused; a freed block is marked */
static int64_t freed;
#define REUSE 0
#else
#define REUSE 1
#endif
static Obj *obj_alloc(void) {
    if (REUSE && slab_free) { Obj *o = slab_free; slab_free = *(Obj **)o; return o; }
    if (slab_cur == slab_end) {
        size_t sz = (size_t)4096 * sizeof(Obj);
        slab_cur = malloc(sz); CHECK(slab_cur, "out of memory");
        slab_end = slab_cur + sz;
    }
    Obj *o = (Obj *)slab_cur; slab_cur += sizeof(Obj); return o;
}
static void obj_free(Obj *o) {
#ifdef POISON
    CHECK(o->id != -1, "poison: a block freed twice");
    o->id = -1; freed++;
    return;
#endif
    *(Obj **)o = slab_free; slab_free = o;
}

/* ---- side records {strong, weak}, reached by the index in the count word ---- */
typedef struct { uint64_t strong, weak; } SideRec;
static SideRec *side; static uint32_t side_len, side_cap, side_free_head = UINT32_MAX;
#ifdef POISON
static uint32_t side_in_use, side_in_use_peak;
#endif
static uint32_t side_alloc(void) {
#ifdef POISON
    if (++side_in_use > side_in_use_peak) side_in_use_peak = side_in_use;
#endif
    if (side_free_head != UINT32_MAX) { uint32_t i = side_free_head; side_free_head = (uint32_t)side[i].weak; return i; }
    if (side_len == side_cap) {
        side_cap = side_cap ? side_cap * 2 : 1024;
        CHECK(side_cap <= LOW, "too many weakly referenced objects");
        side = realloc(side, side_cap * sizeof(SideRec)); CHECK(side, "out of memory");
    }
    return side_len++;
}
static void side_release(uint32_t i) {
#ifdef POISON
    side_in_use--;
#endif
    side[i].weak = side_free_head; side_free_head = i;
}

static void weak_retain(Obj *t) {
    if (!(t->rc & SIDE)) {
        uint32_t i = side_alloc();
        side[i].strong = t->rc & LOW; side[i].weak = 0;
        t->rc = (t->rc & ~LOW) | SIDE | i;
    }
    side[t->rc & LOW].weak++;                    /* 64 bits: cannot overflow in practice */
}
static void weak_release(Obj *t) {
    uint32_t i = t->rc & LOW;
    if (--side[i].weak != 0) return;
    if (t->rc & DEAD) { side_release(i); obj_free(t); }                       /* last reference to a dead object */
    else if (side[i].strong <= LOW) {                                          /* the count goes back to the header */
        t->rc = (t->rc & ~(LOW | SIDE)) | (uint32_t)side[i].strong; side_release(i);
    }
}
static inline void weak_store(Obj **field, Obj *t) {     /* holder.target = t */
    if (t) weak_retain(t);
    Obj *old = *field;
    *field = t;
    if (old) weak_release(old);
}
static inline Obj *weak_read(Obj *t) {
#ifdef POISON
    CHECK(!t || t->id != -1, "poison: a weak field points at a freed block");
#endif
    return (t && !(t->rc & DEAD)) ? t : NULL;
}

static void drop_fields(Obj *o) {
    Obj *t = o->target;
    o->target = NULL;
    if (t) weak_release(t);
}
static void obj_release(Obj *o) {
    if (!(o->rc & SIDE)) {
        if ((--o->rc & LOW) == 0) { drop_fields(o); obj_free(o); }
        return;
    }
    uint32_t i = o->rc & LOW;
    if (--side[i].strong != 0) return;
    o->rc |= DEAD;              /* weak reads now give None; the block waits for its weak count */
    side[i].weak++;             /* hold the block while its own fields drop (they may point at it) */
    drop_fields(o);
    weak_release(o);            /* frees the block now if no weak field points at it */
}

/* ---- the classes' update methods; the type index selects one ---- */
static inline int32_t sign32(int32_t d) { return (d > 0) - (d < 0); }
static void ship_update(Obj *o) {
    Obj *t = weak_read(o->target);
    if (t) { o->x += sign32(t->x - o->x); o->y += sign32(t->y - o->y); }
}
static void rock_update(Obj *o) { o->x = (o->x + 1) % 1000; }      /* x >= 0: floor modulo == C's % */
static void turret_update(Obj *o) {
    Obj *t = weak_read(o->target);
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
#ifdef POISON
    printf("poison check ok: %lld blocks freed, none read after free; side records in use at the end %u, at most %u\n",
           (long long)freed, side_in_use, side_in_use_peak);
#endif
    h_report("combined-w3", (uint64_t)sum + 1000000ull * (uint64_t)destroyed);
    return 0;
}
