/* demo.c: the ECS test's destroy-and-create on four entities, both ways, printing memory after
   every step. EnTT's way uses the same pool functions as bench/w1_group.c, with a counter added to
   every store into a list. Entities: 0 ball (moves), 1 wall (still), 2 rock (moves), 3 cat (moves).
   The job: destroy the rock, create a new moving entity (4). */
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#define NONE 0xFFFFFFFFu
static const char *NAME[] = {"ball", "wall", "rock", "cat", "new"};
static int stores = 0;
#define STORE(lhs, rhs) ((lhs) = (rhs), stores++)

/* ---- EnTT's way (functions as in bench/w1_group.c, stores counted) ---- */
typedef struct { int x, y; } V2;
typedef struct { uint32_t sparse[8]; uint32_t ents[8]; V2 data[8]; size_t n; const char *label; } Pool;
static Pool pos = {.label = "positions"}, vel = {.label = "speeds   "};
static size_t glen;

static void show_pool(const Pool *p) {
    printf("  %s list: [", p->label);
    for (size_t k = 0; k < p->n; k++) printf("%s%s", k ? " | " : "", NAME[p->ents[k]]);
    printf("]   where-is:");
    for (uint32_t id = 0; id < 5; id++) if (p->sparse[id] != NONE) printf(" %s=%u", NAME[id], p->sparse[id]);
    printf("\n");
}
static void show(const char *what) {
    printf("%s  (stores so far: %d; moving group = first %zu)\n", what, stores, glen);
    show_pool(&pos); show_pool(&vel);
}
static int has(const Pool *p, uint32_t id) { return p->sparse[id] != NONE; }
static void add(Pool *p, uint32_t id, V2 v) {
    STORE(p->sparse[id], (uint32_t)p->n); STORE(p->ents[p->n], id); STORE(p->data[p->n], v); p->n++;
}
static void swap_slots(Pool *p, size_t i, size_t j) {
    if (i == j) return;
    uint32_t a = p->ents[i], b = p->ents[j]; V2 t = p->data[i];
    STORE(p->ents[i], b); STORE(p->ents[j], a); STORE(p->data[i], p->data[j]); STORE(p->data[j], t);
    STORE(p->sparse[b], (uint32_t)i); STORE(p->sparse[a], (uint32_t)j);
}
static void del(Pool *p, uint32_t id) {
    uint32_t i = p->sparse[id], last = p->ents[--p->n];
    STORE(p->ents[i], last); STORE(p->data[i], p->data[p->n]); STORE(p->sparse[last], i); STORE(p->sparse[id], NONE);
}
static void group_enter(uint32_t id) { swap_slots(&pos, pos.sparse[id], glen); swap_slots(&vel, vel.sparse[id], glen); glen++; }
static void group_leave(uint32_t id) { glen--; swap_slots(&pos, pos.sparse[id], glen); swap_slots(&vel, vel.sparse[id], glen); }
static void emplace_pos(uint32_t id, V2 v) { add(&pos, id, v); if (has(&vel, id)) group_enter(id); }
static void emplace_vel(uint32_t id, V2 v) { add(&vel, id, v); if (has(&pos, id)) group_enter(id); }

/* ---- the archetype way (as plain/w1.c) ---- */
typedef struct { uint32_t id; int x, y, dx, dy; } Mover;
static Mover mov[8]; static size_t nmov;
static void show_arch(const char *what) {
    printf("%s  (stores so far: %d)\n  movers: [", what, stores);
    for (size_t k = 0; k < nmov; k++) printf("%s%s", k ? " | " : "", NAME[mov[k].id]);
    printf("]   still: [wall]\n");
}

int main(void) {
    for (int i = 0; i < 8; i++) pos.sparse[i] = vel.sparse[i] = NONE;
    uint32_t moving[] = {0, 2, 3};
    emplace_pos(0, (V2){0, 0}); emplace_vel(0, (V2){1, 1});
    emplace_pos(1, (V2){5, 5});
    emplace_pos(2, (V2){2, 2}); emplace_vel(2, (V2){1, 0});
    emplace_pos(3, (V2){3, 3}); emplace_vel(3, (V2){0, 1});
    for (int k = 0; k < 3; k++) mov[nmov++] = (Mover){moving[k], 0, 0, 1, 1};

    printf("=== EnTT's way ===\n");
    stores = 0;
    show("start");
    group_leave(2); show("after group_leave(rock): rock swapped out of the moving group, in both lists");
    del(&pos, 2);   show("after del(positions, rock)");
    del(&vel, 2);   show("after del(speeds, rock)");
    add(&pos, 4, (V2){0, 0}); show("after add(positions, new)");
    add(&vel, 4, (V2){1, 1}); show("after add(speeds, new)");
    group_enter(4); show("after group_enter(new): new swapped into the moving group");
    int entt = stores;

    printf("\n=== the archetype way ===\n");
    stores = 0;
    show_arch("start");
    size_t slot = 1; /* the rock's slot */
    STORE(mov[slot], mov[--nmov]); show_arch("after: the last record (cat) copied over the rock");
    STORE(mov[nmov], ((Mover){4, 0, 0, 1, 1})); nmov++; show_arch("after: the new record written at the end");
    int arch = stores;

    printf("\nstores for destroy rock + create new: EnTT's way %d, archetype way %d\n", entt, arch);
    return 0;
}
