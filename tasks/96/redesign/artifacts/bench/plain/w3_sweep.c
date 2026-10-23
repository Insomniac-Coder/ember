/* VARIANT (superseded by w3.c) (kept for traceability; phase3-measurements.md). Plain C baseline, W3 (game objects that refer to each other). No safety checks. Objects are
   malloc'd and hold a raw target pointer; the update is a call through a per-kind function
   pointer table. `live` holds the live objects in increasing id order. Destroying marks the
   object dead, one sweep over the live objects clears targets that point at dead objects, then
   the dead objects are freed: no pointer dangles. */
#include "../harness.h"
#define M 100000
typedef struct Obj Obj;
typedef struct { void (*update)(Obj *); } VTable;
struct Obj { const VTable *vt; int64_t id; int32_t kind, x, y, health, dead; Obj *target; };

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

static Obj **live, **dead;
static size_t nlive, cap;

static Obj *make(int64_t id) {
    Obj *o = malloc(sizeof(Obj));
    o->id = id; o->kind = (int32_t)(id % 3); o->vt = &VT[o->kind];
    o->x = (int32_t)(h_rand() % 1000); o->y = (int32_t)(h_rand() % 1000);
    o->health = (int32_t)(100 + id % 50); o->dead = 0;
    return o;
}
static void push(Obj *o) {
    if (nlive == cap) { cap *= 2; live = realloc(live, sizeof(Obj *) * cap); dead = realloc(dead, sizeof(Obj *) * cap); }
    live[nlive++] = o;
}

int main(void) {
    h_start();
    h_seed(7);
    cap = M; live = malloc(sizeof(Obj *) * cap); dead = malloc(sizeof(Obj *) * cap);
    for (int64_t i = 0; i < M; i++) {
        Obj *o = make(i);
        o->target = i == 0 ? NULL : live[h_rand() % (uint64_t)i];
        push(o);
    }
    int64_t next = M;
    uint64_t total = 0;
    for (int f = 0; f < 1000; f++) {
        for (size_t k = 0; k < nlive; k++) live[k]->vt->update(live[k]);
        size_t nd = 0, j = 0;
        for (size_t k = 0; k < nlive; k++) {
            Obj *o = live[k];
            if (o->health <= 0) { o->dead = 1; dead[nd++] = o; } else live[j++] = o;
        }
        nlive = j;
        if (nd) {
            for (size_t k = 0; k < nlive; k++)
                if (live[k]->target && live[k]->target->dead) live[k]->target = NULL;
            for (size_t d = 0; d < nd; d++) free(dead[d]);
        }
        total += nd;
        for (size_t d = 0; d < nd; d++) {
            Obj *o = make(next++);
            o->target = nlive ? live[h_rand() % nlive] : NULL;
            push(o);
        }
    }
    int64_t sum = 0;
    for (size_t k = 0; k < nlive; k++) sum += (int64_t)live[k]->x + live[k]->y + live[k]->health;
    h_report("plain-w3-sweep", (uint64_t)sum + 1000000ull * total);
    return 0;
}
