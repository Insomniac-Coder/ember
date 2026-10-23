/* D1 (values and projections), workload W3: game objects that refer to each other.
   Report: tasks/96/redesign/phase3-design-D1.md (sections 3, 4, 5).

   The user program declares `open class Thing` with `weak target: Option[Thing]` and a virtual
   `update`, overridden by Ship, Rock and Turret; `live: Array[Thing]` holds the objects.
   What D1 needs and is therefore here:
   - a class object is one heap block: 16-byte header (strong u32, weak u32, type table pointer),
     then its fields; `weak` starts at 1 on behalf of all strong handles ([OBJ-3]);
   - a virtual call loads the method from the object's type table;
   - reading a weak field tests the target's strong count; a dead target is cleared from the field
     (lazy zeroing) and its weak count released; storing one adjusts weak counts;
   - the handle the update loop yields is not counted: `update` cannot reach `live`, a local
     ([RC-2e]); the strong handle a weak read gives is not counted either: nothing between the
     read and its last use can release anything;
   - `live.retain(...)` tests uniqueness, compacts in place, releases the handles it drops (a
     dropped object releases its own weak target, then its weak unit), bumps the shift version;
   - an Array value is one pointer to a counted buffer; push tests count and capacity; indices the
     compiler cannot prove in range are checked (h_rand() % i against live.len()).
   Left out on purpose, the same in every memory model: the integer overflow checks of [TYP-8].
   No teardown at the end: memory-only values whose last use ends main are not freed. */
#include "../harness.h"
#include <string.h>

static _Noreturn void d1_panic(const char *msg) {
    fprintf(stderr, "panic: %s\n", msg);
    fflush(stderr);
    ExitProcess(101);
}
static void *xmalloc(size_t n) {
    void *p = malloc(n);
    if (!p) d1_panic("out of memory");
    return p;
}

/* ---- D1 runtime: counted buffers (Array) ---- */
typedef struct Buf { uint32_t count; uint32_t shifts; uint64_t len; uint64_t cap; } Buf; /* 24 bytes */
static Buf EMPTY = {0x80000000u, 0, 0, 0};
#define EL(b, T) ((T *)((Buf *)(b) + 1))
typedef void (*RetainFn)(void *elems, uint64_t n);

static Buf *buf_reserve_slow(Buf **slot, size_t esz, uint64_t need, RetainFn retain) {
    Buf *b = *slot;
    uint64_t cap = b->cap;
    if (need > cap) {
        uint64_t nc = cap < 4 ? 4 : cap * 2;
        while (nc < need) nc *= 2;
        cap = nc;
    }
    if (b->count == 1) {
        b = realloc(b, sizeof(Buf) + cap * esz);
        if (!b) d1_panic("out of memory");
        b->cap = cap;
    } else {
        Buf *nb = xmalloc(sizeof(Buf) + cap * esz);
        nb->count = 1; nb->shifts = 0; nb->len = b->len; nb->cap = cap;
        memcpy(nb + 1, b + 1, b->len * esz);
        if (retain) retain(nb + 1, b->len);
        if (b != &EMPTY) b->count--;
        b = nb;
    }
    *slot = b;
    return b;
}
static inline Buf *buf_mut(Buf **slot, size_t esz, RetainFn retain) {
    Buf *b = *slot;
    if (__builtin_expect(b->count != 1, 0)) b = buf_reserve_slow(slot, esz, b->len, retain);
    return b;
}
#define PUSH(slot, T, val, retain)                                                                   \
    do {                                                                                             \
        Buf *b_ = *(slot);                                                                           \
        if (__builtin_expect(b_->count != 1 || b_->len == b_->cap, 0))                               \
            b_ = buf_reserve_slow((slot), sizeof(T), b_->len + 1, (retain));                         \
        EL(b_, T)[b_->len++] = (val);                                                                \
    } while (0)

/* ---- open class Thing and its three subclasses ---- */
typedef struct Thing Thing;
typedef struct VT { const char *name; void (*update)(Thing *); } VT; /* type info with the vtable */
struct Thing {
    uint32_t strong, weak;  /* header */
    const VT *type;         /* header */
    int64_t id;             /* let id: int */
    int32_t x, y, health;
    Thing *target;          /* weak target: Option[Thing] */
};                          /* 48 bytes */

static void retain_things(void *el, uint64_t n) {
    Thing **a = el;
    for (uint64_t i = 0; i < n; i++) a[i]->strong++;
}
static inline void weak_release(Thing *t) {
    if (--t->weak == 0) free(t);
}
static inline void thing_release(Thing *t) {
    if (--t->strong == 0) { /* deinit: drop the fields (the weak target), then the weak unit */
        if (t->target) weak_release(t->target);
        weak_release(t);
    }
}
static inline Thing *weak_get(Thing **field) {
    Thing *p = *field;
    if (p && __builtin_expect(p->strong == 0, 0)) {
        *field = NULL;
        weak_release(p);
        p = NULL;
    }
    return p;
}
static inline void weak_set(Thing **field, Thing *v) {
    Thing *old = *field;
    if (v) v->weak++;
    *field = v;
    if (old) weak_release(old);
}
static inline int32_t sign32(int32_t v) { return (v > 0) - (v < 0); }

static void ship_update(Thing *o) { /* if Some(t) = self.target: self.x += sign(t.x - self.x) ... */
    Thing *t = weak_get(&o->target);
    if (t) {
        o->x += sign32(t->x - o->x);
        o->y += sign32(t->y - o->y);
    }
}
static void rock_update(Thing *o) { o->x = (o->x + 1) % 1000; } /* x >= 0: floor mod == C % */
static void turret_update(Thing *o) {
    Thing *t = weak_get(&o->target);
    if (t && abs(t->x - o->x) + abs(t->y - o->y) < 300) t->health -= 1;
}
static const VT VT_SHIP = {"Ship", ship_update};
static const VT VT_ROCK = {"Rock", rock_update};
static const VT VT_TURRET = {"Turret", turret_update};

/* fn make(id) -> Thing: x = h_rand() % 1000; y = h_rand() % 1000; match id % 3: ... */
static Thing *make_thing(int64_t id) {
    int32_t x = (int32_t)(h_rand() % 1000);
    int32_t y = (int32_t)(h_rand() % 1000);
    Thing *t = xmalloc(sizeof *t);
    t->strong = 1; t->weak = 1;
    int64_t kind = id % 3;
    t->type = kind == 0 ? &VT_SHIP : kind == 1 ? &VT_ROCK : &VT_TURRET;
    t->id = id; t->x = x; t->y = y; t->health = (int32_t)(100 + id % 50);
    t->target = NULL;
    return t;
}

int main(void) {
    h_start();
    h_seed(7);
    enum { M = 100000, FRAMES = 1000 };
    Buf *live = &EMPTY;

    for (int64_t i = 0; i < M; i++) {
        Thing *t = make_thing(i);
        if (i > 0) {
            uint32_t r = h_rand() % (uint32_t)i;
            if (r >= live->len) d1_panic("index out of bounds");
            weak_set(&t->target, EL(live, Thing *)[r]);
        }
        PUSH(&live, Thing *, t, retain_things); /* last use of t: a move */
    }

    int64_t next_id = M, destroyed = 0;
    for (int f = 0; f < FRAMES; f++) {
        /* (a) `for t in live: t.update()` */
        const Buf *lb = live;
        for (uint64_t i = 0; i < lb->len; i++) {
            Thing *t = EL(lb, Thing *)[i];
            t->type->update(t);
        }
        /* (b) `before = live.len(); live.retain(fn(t) => t.health > 0)` */
        Buf *mb = buf_mut(&live, sizeof(Thing *), retain_things);
        uint64_t before = mb->len, w = 0;
        for (uint64_t r = 0; r < before; r++) {
            Thing *t = EL(mb, Thing *)[r];
            if (t->health > 0) EL(mb, Thing *)[w++] = t;
            else thing_release(t);
        }
        mb->len = w;
        if (w != before) mb->shifts++;
        int64_t dead = (int64_t)(before - w);
        destroyed += dead;
        /* (c) `for _ in 0..dead: t = make(next_id); ...; if live.len() > 0: t.target = live[h_rand() % live.len()]` */
        for (int64_t d = 0; d < dead; d++) {
            Thing *t = make_thing(next_id);
            next_id += 1;
            uint64_t n = live->len;
            if (n > 0) {
                uint64_t r = h_rand() % n; /* < live.len(): proven */
                weak_set(&t->target, EL(live, Thing *)[r]);
            }
            PUSH(&live, Thing *, t, retain_things);
        }
    }

    int64_t total = destroyed * 1000000;
    for (uint64_t i = 0; i < live->len; i++) {
        const Thing *t = EL(live, Thing *)[i];
        total += (int64_t)t->x + t->y + t->health;
    }
    h_report("D1-w3", (uint64_t)total);
    return 0;
}
