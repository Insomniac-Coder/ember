/* D2 (generational references + stable stores), workload W3: game objects that refer to each other.
   What the D2 compiler would emit for tasks/96/redesign/phase3-design-D2.md section 3, W3:
   - Ship, Rock and Turret share their root class's store: 40-byte slots in a reserved address range
     that never move, {u32 generation, u32 class id, fields}; slot 0 is reserved so None = {0, 0};
   - a reference is 8 bytes, {slot index, generation}; `objects: list[own Object]` owns every object;
   - update() is a virtual call through the class id's table entry;
   - `self` and the elements of `objects` are read from their owner: no check (proof P1);
   - `target: Option[Object]` is a reference; reading it checks the target's generation, and a
     destroyed target reads as None (this check is the program's meaning: it is never removed);
   - `objects.remove(o)` inside `for o in objects` destroys o at once (its generation changes, so every
     target that refers to it reads None from then on); the list closes the gap when the loop ends;
   - destroyed slots go on a free list and are reused (LIFO) by the objects created in (c).
   Not modelled (outside the memory model, same for every design): integer overflow checks. */
#include "../bench/harness.h"
#include <string.h>

typedef uint32_t u32;
typedef int32_t i32;
typedef uint64_t u64;
typedef int64_t i64;

#define M 100000u
#define FRAMES 1000u
#define NO_SLOT UINT32_MAX

static void d2_panic(const char *m) {
    fprintf(stderr, "panic: %s\n", m);
    fflush(stderr);
    ExitProcess(101);
}

typedef struct { char *base; size_t reserved, committed; } Range;

static void range_init(Range *r, size_t reserve) {
    r->base = (char *)VirtualAlloc(NULL, reserve, MEM_RESERVE, PAGE_NOACCESS);
    if (!r->base) d2_panic("address space reservation failed");
    r->reserved = reserve;
    r->committed = 0;
}

static void range_ensure(Range *r, size_t bytes) {
    if (bytes <= r->committed) return;
    size_t chunk = (size_t)1 << 20;
    size_t want = (bytes + chunk - 1) & ~(chunk - 1);
    if (want > r->reserved) d2_panic("store capacity exhausted");
    if (!VirtualAlloc(r->base + r->committed, want - r->committed, MEM_COMMIT, PAGE_READWRITE))
        d2_panic("out of memory");
    r->committed = want;
}

typedef struct { u32 idx, gen; } Ref; /* None = {0, 0} */

typedef struct {
    u32 gen;    /* odd = live, even = dead */
    u32 cls;    /* 0 Ship, 1 Rock, 2 Turret; holds the free-list link while dead */
    i64 id;
    i32 x, y, health;
    u32 pad;
    Ref target; /* Option[Object] */
} ObjSlot;      /* 40 bytes */

typedef struct { Range r; u32 len; u32 free_top; } Store;
static Store objs;

static inline ObjSlot *obj(u32 idx) { return (ObjSlot *)objs.r.base + idx; }

static void store_init(void) {
    range_init(&objs.r, (size_t)256 << 20);
    range_ensure(&objs.r, sizeof(ObjSlot));
    obj(0)->gen = 2; /* reserved: never allocated, never equal to None's 0 */
    objs.len = 1;
    objs.free_top = NO_SLOT;
}

static Ref obj_alloc(void) {
    u32 s = objs.free_top;
    if (s != NO_SLOT) {
        objs.free_top = obj(s)->cls;
    } else {
        s = objs.len++;
        range_ensure(&objs.r, (size_t)objs.len * sizeof(ObjSlot));
    }
    ObjSlot *o = obj(s);
    o->gen += 1; /* odd */
    return (Ref){s, o->gen};
}

static void obj_destroy(u32 s) {
    ObjSlot *o = obj(s);
    o->gen += 1;                    /* even: every reference to it is now stale */
    if (o->gen == UINT32_MAX - 1) return; /* generation exhausted: the slot is retired */
    o->cls = objs.free_top;
    objs.free_top = s;
}

typedef struct { Ref *ptr; i64 len, cap; } RefList;
typedef struct { i64 *ptr; i64 len, cap; } IntList;

static void reflist_push(RefList *l, Ref r) {
    if (l->len == l->cap) {
        i64 nc = l->cap ? l->cap * 2 : 4;
        Ref *np = (Ref *)realloc(l->ptr, (size_t)nc * sizeof(Ref));
        if (!np) d2_panic("out of memory");
        l->ptr = np;
        l->cap = nc;
    }
    l->ptr[l->len++] = r;
}

static void intlist_push(IntList *l, i64 v) {
    if (l->len == l->cap) {
        i64 nc = l->cap ? l->cap * 2 : 4;
        i64 *np = (i64 *)realloc(l->ptr, (size_t)nc * sizeof(i64));
        if (!np) d2_panic("out of memory");
        l->ptr = np;
        l->cap = nc;
    }
    l->ptr[l->len++] = v;
}

static inline i32 sign(i32 d) { return (d > 0) - (d < 0); }

/* the three overrides of Object.update; `self` arrives live from its owner (P1) */
static void ship_update(Ref self) {
    ObjSlot *o = obj(self.idx);
    Ref t = o->target;
    ObjSlot *to = obj(t.idx);
    if (to->gen == t.gen) { /* `if t is not None`: a destroyed target reads None */
        o->x += sign(to->x - o->x);
        o->y += sign(to->y - o->y);
    }
}

static void rock_update(Ref self) {
    ObjSlot *o = obj(self.idx);
    o->x = (o->x + 1) % 1000;
}

static void turret_update(Ref self) {
    ObjSlot *o = obj(self.idx);
    Ref t = o->target;
    ObjSlot *to = obj(t.idx);
    if (to->gen == t.gen) {
        if (abs(to->x - o->x) + abs(to->y - o->y) < 300) to->health -= 1;
    }
}

typedef void (*UpdateFn)(Ref);
static UpdateFn const update_table[3] = {ship_update, rock_update, turret_update};

static Ref object_new(i64 id, i32 x, i32 y, Ref target) {
    Ref r = obj_alloc();
    ObjSlot *o = obj(r.idx);
    o->cls = (u32)(id % 3);
    o->id = id;
    o->x = x;
    o->y = y;
    o->health = (i32)(100 + id % 50);
    o->target = target;
    return r;
}

int main(void) {
    h_start();
    h_seed(7);
    store_init();

    RefList objects = {0}; /* objects: list[own Object], in increasing id order */
    for (u32 i = 0; i < M; i++) {
        i32 x = (i32)(h_rand() % 1000);
        i32 y = (i32)(h_rand() % 1000);
        Ref t = {0, 0};
        if (i != 0) t = objects.ptr[h_rand() % i]; /* in range: objects holds i elements */
        reflist_push(&objects, object_new(i, x, y, t));
    }
    i64 next_id = M;
    u64 destroyed_total = 0;
    IntList dead = {0};

    for (u32 f = 0; f < FRAMES; f++) {
        /* (a) for o in objects: o.update() */
        for (i64 i = 0; i < objects.len; i++) {
            Ref r = objects.ptr[i];
            update_table[obj(r.idx)->cls](r);
        }

        /* (b) for o in objects: if o.health <= 0: dead.append(o.id); objects.remove(o) */
        dead.len = 0;
        i64 first_removed = -1;
        for (i64 i = 0; i < objects.len; i++) {
            Ref r = objects.ptr[i];
            ObjSlot *o = obj(r.idx);
            if (o->health <= 0) {
                intlist_push(&dead, o->id);
                obj_destroy(r.idx);
                objects.ptr[i] = (Ref){0, 0}; /* removed: the gap is closed when the loop ends */
                if (first_removed < 0) first_removed = i;
            }
        }
        if (first_removed >= 0) {
            i64 w = first_removed;
            for (i64 i = first_removed; i < objects.len; i++)
                if (objects.ptr[i].gen != 0) objects.ptr[w++] = objects.ptr[i];
            objects.len = w;
        }
        destroyed_total += (u64)dead.len;

        /* (c) for _ in dead: objects.append(Object.make(next_id, ...)) */
        for (i64 k = 0; k < dead.len; k++) {
            i64 id = next_id++;
            i32 x = (i32)(h_rand() % 1000);
            i32 y = (i32)(h_rand() % 1000);
            Ref t = {0, 0};
            if (objects.len > 0) t = objects.ptr[h_rand() % (u32)objects.len];
            reflist_push(&objects, object_new(id, x, y, t));
        }
    }

    i64 s = 0;
    for (i64 i = 0; i < objects.len; i++) {
        ObjSlot *o = obj(objects.ptr[i].idx);
        s += (i64)o->x + (i64)o->y + (i64)o->health;
    }
    u64 sum = (u64)s + 1000000ull * destroyed_total;

    h_report("D2-w3", sum);
    return 0;
}
