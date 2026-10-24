/* share_idx.c: the ownership group's shared-object workload (target/redesign2/ownership/bench/share.c)
 * with the players in one array and each watcher holding a position instead of a pointer: what a C
 * programmer writing for speed does, and what Ember can write today with a struct Player in an Array.
 * Same work, same random numbers, same checksum as share.c. Build: clang -O2 share_idx.c -o share_idx.exe */
#include "../../../redesign/bench/harness.h"
#include <string.h>
#define P_COUNT (1u << 18)
#define W_COUNT (1u << 20)
#define FRAMES 32
typedef struct { int32_t *p; size_t len, cap; } IVec;
static __declspec(noinline) void ivec_grow(IVec *v) {
    size_t cap = v->cap ? v->cap * 2 : 4;
    int32_t *p = (int32_t *)realloc(v->p, cap * sizeof(int32_t));
    if (!p) abort();
    v->p = p; v->cap = cap;
}
static inline void ivec_push(IVec *v, int32_t x) { if (v->len == v->cap) ivec_grow(v); v->p[v->len++] = x; }
typedef struct { int32_t hp; int32_t x; IVec inv; } Player;   /* 32 bytes, stored in one array */
int main(void) {
    h_start();
    h_seed(42);
    Player *players = (Player *)malloc(P_COUNT * sizeof(Player));
    uint32_t *ws = (uint32_t *)malloc(W_COUNT * sizeof(uint32_t));
    if (!players || !ws) abort();
    memset(players, 0, P_COUNT * sizeof(Player));
    for (uint32_t i = 0; i < P_COUNT; i++) { players[i].hp = 1000; players[i].x = (int32_t)(i & 1023); }
    for (uint32_t i = 0; i < W_COUNT; i++) ws[i] = h_rand() & (P_COUNT - 1);
    uint64_t acc = 0;
    for (uint32_t f = 0; f < FRAMES; f++) {
        for (uint32_t i = 0; i < W_COUNT; i++) {
            Player *t = &players[ws[i]];
            t->hp -= 1;
            acc += (uint64_t)(uint32_t)t->x * (uint64_t)(t->inv.len + 1);
            if ((i & 7) == (f & 7)) {
                ivec_push(&t->inv, (int32_t)f);
                if (t->inv.len > 8) {
                    for (size_t k = 0; k < t->inv.len; k++) acc += (uint64_t)(uint32_t)t->inv.p[k];
                    t->inv.len = 0;
                }
            }
            if ((h_rand() & 15) == 0) ws[i] = h_rand() & (P_COUNT - 1);
        }
    }
    uint64_t sum = acc;
    for (uint32_t i = 0; i < P_COUNT; i++) sum += (uint64_t)(uint32_t)players[i].hp + players[i].inv.len;
    h_report("share-plain-index", sum);
    return 0;
}
