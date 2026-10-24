/* state.c: A23, an NPC state machine whose transitions carry the owned path into the next state.
   This is plain C, and it is also the C the proposed Ember rule emits for
       match owned self.state:
           State.Walking(path): ... self.state = State.Running(path)
   the payload's bits are read out, the field is a hole until the one store that refills it, and no
   placeholder is written (Rust's mem::replace idiom writes one; see state_rust.rs). */
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
        Vec32 path = n->state.path;                    /* moved out: n->state is a hole */
        path.len--; *sum += (uint32_t)path.p[path.len]; /* path.pop() */
        if (path.len == 0) { free(path.p); n->state.tag = IDLE; }
        else if ((i + tick) % 3 == 0) { n->state.tag = RUNNING; n->state.path = path; }
        else { n->state.tag = WALKING; n->state.path = path; }
        break;
    }
    case RUNNING: {
        Vec32 path = n->state.path;
        *sum += path.len;
        n->state.tag = WALKING; n->state.path = path;
        break;
    }
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
    h_report("state-c", sum);
    return 0;
}
