/* D3 model of W3 (game objects that refer to each other), as the D3 compiler would emit the program
   in tasks/96/redesign/phase3-design-D3.md (section 3, W3).

   What D3 puts here, and nothing else:
   - `objects` (a List local to main) is the HOME of every object; `target` is a WATCHING reference.
     `objects` removes objects, so a watcher can outlive its target: every `target` is checked
     (pointer + generation; the object keeps its generation in its header). A destroyed object's
     generation is bumped and its slot goes back to the object slab's free list for this size class
     (the generation word is never overwritten; an exhausted slot is retired). Reading a dead target
     gives None, as the program's `if t := self.target:` asks.
   - Object header: generation (u32, for the watchers) + type index (u32, for the virtual call).
     The watcher's generation sits in the padding after the int32 fields: 40 bytes per object.
   - (a) The update loop changes nothing structural in `objects` (whole-program effect fact: no
     update can remove from or add to it): a plain loop, no pin, `self` unchecked.
   - (b) The destroy loop removes the loop's own element: `objects` is a local, so the pin is static
     (no counter): the slot becomes a tombstone (NULL), the object is destroyed at once (it is not
     used again), and the list is compacted, in order, when the loop ends.
   - Checks kept: generation compares on every target use, the bounds check on `objects[h_rand() % i]`
     in the first loop (not provable), the tombstone test in the pinned loop.
   Left out on purpose, identical in every memory model: integer overflow checks ([TYP-8]) and the
   floor-modulo sign fix-up for `x % 1000` (x stays in 0..999 here; the result is the same). */
#include "../sh.h"
#include <string.h>

#define M 100000
#define FRAMES 1000

static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)
static void *xrealloc(void *p, size_t n) { void *q = realloc(p, n); CHECK(q, "out of memory"); return q; }

enum { SHIP = 0, ROCK = 1, TURRET = 2 }; /* type indexes of the three classes */

typedef struct Obj Obj;
struct Obj {
    uint32_t gen;          /* header: generation */
    uint32_t type;         /* header: type index */
    int64_t id;
    int32_t x, y, health;
    uint32_t target_gen;   /* watcher: generation part */
    Obj *target;           /* watcher: pointer part, NULL = None */
};
_Static_assert(sizeof(Obj) == 40, "object layout");

/* the runtime's object slab for this size class: bump, then a free list; freed slots keep `gen` */
static char *slab_cur, *slab_end;
static Obj *free_list;
static Obj *obj_alloc(void) {
    Obj *o;
    if (free_list) {
        o = free_list;
        free_list = o->target;  /* the free-list link lives in the payload, never in `gen` */
    } else {
        if (slab_cur == slab_end) {
            size_t sz = (size_t)4096 * sizeof(Obj);
            slab_cur = xrealloc(NULL, sz);
            slab_end = slab_cur + sz;
        }
        o = (Obj *)slab_cur;
        slab_cur += sizeof(Obj);
        o->gen = 1;
    }
    return o;
}
static void obj_destroy(Obj *o) {           /* no drop code in these classes */
    if (++o->gen == 0) return;              /* generation exhausted: retire the slot */
    o->target = free_list;
    free_list = o;
}
static inline Obj *watch(Obj *p, uint32_t g) { return (p && p->gen == g) ? p : NULL; }

static inline int32_t sign(int32_t d) { return (d > 0) - (d < 0); }

static void ship_update(Obj *self) {
    Obj *t = watch(self->target, self->target_gen);
    if (t) {
        self->x += sign(t->x - self->x);
        self->y += sign(t->y - self->y);
    }
}
static void rock_update(Obj *self) { self->x = (self->x + 1) % 1000; }
static void turret_update(Obj *self) {
    Obj *t = watch(self->target, self->target_gen);
    if (t && abs(t->x - self->x) + abs(t->y - self->y) < 300) t->health -= 1;
}
static void (*const UPDATE[3])(Obj *) = { ship_update, rock_update, turret_update };

typedef struct { Obj **data; int64_t len, cap; } ListObj;   /* a local List: plain header */
typedef struct { int64_t *data; int64_t len, cap; } ListI64;
static inline void lo_push(ListObj *l, Obj *o) {
    if (l->len == l->cap) { int64_t c = l->cap ? l->cap * 2 : 4; l->data = xrealloc(l->data, (size_t)c * 8); l->cap = c; }
    l->data[l->len++] = o;
}
static inline void li_push(ListI64 *l, int64_t v) {
    if (l->len == l->cap) { int64_t c = l->cap ? l->cap * 2 : 4; l->data = xrealloc(l->data, (size_t)c * 8); l->cap = c; }
    l->data[l->len++] = v;
}

/* make(kind, id, x, y, health, target): Ship, Rock or Turret */
static Obj *make(int64_t kind, int64_t id, int32_t x, int32_t y, int32_t health, Obj *target) {
    Obj *o = obj_alloc();
    o->type = (uint32_t)kind;
    o->id = id; o->x = x; o->y = y; o->health = health;
    o->target = target;
    o->target_gen = target ? target->gen : 0;   /* storing a watcher reads the target's generation */
    return o;
}

int main(void) {
    h_start();
    h_seed(7);
    ListObj objects = {0};
    for (int64_t i = 0; i < M; i++) {
        int32_t x = (int32_t)(h_rand() % 1000);
        int32_t y = (int32_t)(h_rand() % 1000);
        Obj *target = NULL;
        if (i != 0) {
            int64_t k = (int64_t)(h_rand() % (uint64_t)i);
            CHECK(k < objects.len, "index out of bounds");
            target = objects.data[k];
        }
        lo_push(&objects, make(i % 3, i, x, y, (int32_t)(100 + i % 50), target));
    }

    int64_t next_id = M, total_destroyed = 0;
    ListI64 destroyed = {0};
    for (int frame = 0; frame < FRAMES; frame++) {
        /* (a) for o in objects: o.update() */
        Obj **od = objects.data;
        int64_t n = objects.len;
        for (int64_t i = 0; i < n; i++) {
            Obj *o = od[i];
            UPDATE[o->type](o);
        }
        /* (b) for o in objects: if o.health <= 0: destroyed.append(o.id); objects.remove(o) */
        destroyed.len = 0;
        int64_t first_tomb = -1;
        for (int64_t i = 0; i < n; i++) {
            Obj *o = od[i];
            if (!o) continue;                      /* tombstone test of the pinned loop */
            if (o->health <= 0) {
                li_push(&destroyed, o->id);
                od[i] = NULL;                      /* remove the current element: a tombstone */
                if (first_tomb < 0) first_tomb = i;
                obj_destroy(o);                    /* its home let it go and `o` is not used again */
            }
        }
        if (first_tomb >= 0) {                     /* the loop ends: compact, keeping the order */
            int64_t j = first_tomb;
            for (int64_t i = first_tomb + 1; i < n; i++)
                if (od[i]) od[j++] = od[i];
            objects.len = j;
        }
        total_destroyed += destroyed.len;
        /* (c) for _ in destroyed: ... objects.append(make(...)) */
        for (int64_t d = 0; d < destroyed.len; d++) {
            int64_t id = next_id++;
            int32_t x = (int32_t)(h_rand() % 1000);
            int32_t y = (int32_t)(h_rand() % 1000);
            Obj *target = NULL;
            if (objects.len > 0) target = objects.data[(int64_t)h_rand() % objects.len]; /* x % len < len: proven */
            lo_push(&objects, make(id % 3, id, x, y, (int32_t)(100 + id % 50), target));
        }
    }

    int64_t sum = 0;
    for (int64_t i = 0; i < objects.len; i++) {
        Obj *o = objects.data[i];
        sum += (int64_t)o->x + o->y + o->health;
    }
    h_report("d3-w3", (uint64_t)sum + 1000000ull * (uint64_t)total_destroyed);
    return 0;
}
