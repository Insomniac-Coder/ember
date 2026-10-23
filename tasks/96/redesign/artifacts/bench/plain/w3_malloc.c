/* VARIANT (superseded by w3.c). Plain C baseline, W3 (game objects that refer to each other). No safety checks. Objects are
   malloc'd and hold a raw target pointer; the update is a call through the object's vtable.
   `live` holds the live objects in increasing id order. Every object keeps an intrusive list of
   the objects that target it (the classic observer list), so destroying an object clears exactly
   its watchers' targets and then frees it: no pointer dangles, and nothing is swept.
   (A first version swept every live object each frame instead: w3_sweep.c.) */
#include "../harness.h"
#define M 100000
typedef struct Obj Obj;
typedef struct { void (*update)(Obj *); } VTable;
struct Obj {
    const VTable *vt; int64_t id; int32_t x, y, health;
    Obj *target;               /* what this object targets, or NULL */
    Obj *watchers;             /* head of the list of objects whose target is this one */
    Obj *wnext, *wprev;        /* this object's links in its target's watcher list */
};

static int sgn(int32_t d) { return (d > 0) - (d < 0); }
static void ship(Obj *o) {
    Obj *t = o->target;
    if (t) { o->x += sgn(t->x - o->x); o->y += sgn(t->y - o->y); }
}
static void rock(Obj *o) { o->x = (o->x + 1) % 1000; }
static void turret(Obj *o) {
    Obj *t = o->target;
    if (t && abs(t->x - o->x) + abs(t->y - o->y) < 300) t->health -= 1;
}
static const VTable VT[3] = {{ship}, {rock}, {turret}};

static Obj **live;
static size_t nlive, cap;

static Obj *make(int64_t id) {
    Obj *o = malloc(sizeof(Obj));
    o->id = id; o->vt = &VT[id % 3];
    o->x = (int32_t)(h_rand() % 1000); o->y = (int32_t)(h_rand() % 1000);
    o->health = (int32_t)(100 + id % 50);
    o->target = o->watchers = o->wnext = o->wprev = NULL;
    return o;
}
static void set_target(Obj *o, Obj *t) {
    o->target = t;
    if (!t) return;
    o->wprev = NULL; o->wnext = t->watchers;
    if (t->watchers) t->watchers->wprev = o;
    t->watchers = o;
}
static void destroy(Obj *o) {
    for (Obj *w = o->watchers, *n; w; w = n) { n = w->wnext; w->target = NULL; }
    Obj *t = o->target;
    if (t) {
        if (o->wprev) o->wprev->wnext = o->wnext; else t->watchers = o->wnext;
        if (o->wnext) o->wnext->wprev = o->wprev;
    }
    free(o);
}
static void push(Obj *o) {
    if (nlive == cap) { cap *= 2; live = realloc(live, sizeof(Obj *) * cap); }
    live[nlive++] = o;
}

int main(void) {
    h_start();
    h_seed(7);
    cap = M; live = malloc(sizeof(Obj *) * cap);
    for (int64_t i = 0; i < M; i++) {
        Obj *o = make(i);
        set_target(o, i == 0 ? NULL : live[h_rand() % (uint64_t)i]);
        push(o);
    }
    int64_t next = M;
    uint64_t total = 0;
    for (int f = 0; f < 1000; f++) {
        for (size_t k = 0; k < nlive; k++) live[k]->vt->update(live[k]);
        size_t nd = 0, j = 0;
        for (size_t k = 0; k < nlive; k++) {
            Obj *o = live[k];
            if (o->health <= 0) { destroy(o); nd++; } else live[j++] = o;
        }
        nlive = j;
        total += nd;
        for (size_t d = 0; d < nd; d++) {
            Obj *o = make(next++);
            set_target(o, nlive ? live[h_rand() % nlive] : NULL);
            push(o);
        }
    }
    int64_t sum = 0;
    for (size_t k = 0; k < nlive; k++) sum += (int64_t)live[k]->x + live[k]->y + live[k]->health;
    h_report("plain-w3-malloc", (uint64_t)sum + 1000000ull * total);
    return 0;
}
