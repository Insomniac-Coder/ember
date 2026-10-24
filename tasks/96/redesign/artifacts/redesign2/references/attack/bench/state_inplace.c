/* state_inplace.c: A23 as the saved answer (7.4) emits it. `path` is bound as `ref mut` (it is written through),
   `pop` works in place (with Ember's emptiness check), and `self.state = State.Running(path)` moves the payload
   out and stores the new state; Walking and Running keep `path` at the same offset, so only the tag changes.
   Compare state.c / state_checked.c (../../bench), which read the whole payload out and write it back every tick. */
#include "harness.h"
#define NNPC 1000000u
#define TICKS 100u
typedef struct { int32_t *p; size_t len, cap; } Vec32;
enum { IDLE = 0, WALKING = 1, RUNNING = 2 };
typedef struct { uint32_t tag; Vec32 path; } State;
typedef struct { State state; } Npc;

static inline void update(Npc *n, uint32_t i, uint32_t tick, uint64_t *sum) {
    switch (n->state.tag) {
    case WALKING: {
        Vec32 *path = &n->state.path;                       /* a `ref mut` binding */
        if (path->len == 0) { fputs("panic: pop of an empty Array", stderr); exit(101); }
        path->len--; *sum += (uint32_t)path->p[path->len];  /* path.pop() in place */
        if (path->len == 0) { free(path->p); n->state.tag = IDLE; }   /* the overwrite drops the path, stores Idle */
        else if ((i + tick) % 3 == 0) n->state.tag = RUNNING;         /* State.Running(path): the payload stays */
        break;
    }
    case RUNNING:
        *sum += n->state.path.len;
        n->state.tag = WALKING;
        break;
    default:
        *sum += 1;
    }
}

int main(void) {
    h_start(); h_seed(11);
    Npc *npcs = (Npc *)malloc(NNPC * sizeof(Npc));
    for (uint32_t i = 0; i < NNPC; i++) {
        size_t n = 1 + h_rand() % 32;
        int32_t *p = (int32_t *)malloc(n * sizeof(int32_t));
        for (size_t k = 0; k < n; k++) p[k] = (int32_t)(h_rand() % 1000);
        npcs[i].state.tag = WALKING; npcs[i].state.path = (Vec32){p, n, n};
    }
    uint64_t sum = 0;
    for (uint32_t t = 0; t < TICKS; t++)
        for (uint32_t i = 0; i < NNPC; i++) update(&npcs[i], i, t, &sum);
    for (uint32_t i = 0; i < NNPC; i++) if (npcs[i].state.tag != IDLE) free(npcs[i].state.path.p);
    free(npcs);
    h_report("state-inplace", sum);
    return 0;
}
