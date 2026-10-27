/* plain_this_tick.c: workload this_tick, plain C. AN-17's `Game.tick` (run4/v5/area-B.md line 190):
   every frame each player loses `damage` hit points; a dead player is counted, respawned and noted
   in the frame's log; the game sums the players' hit points.

   What a careful C programmer writes: one function that takes `Game *`, fields read where the
   method reads them, the walk's list words read once (as the model's walk does). No checks.
   -DNOALIAS_THIS puts `restrict` on the object pointer (what Rust's `&mut self` gives LLVM).

   Layout: the model's (a list of pointers to players carved in order from one block, as a slab
   gives class objects), without the model's 8-byte class header. */
#include "m4.h"

#ifdef NOALIAS_THIS
#define THIS RESTRICT
#define NAME "plain_this_tick_restrict"
#else
#define THIS
#define NAME "plain_this_tick"
#endif

#define PLAYERS 16384
#define FRAMES 12000

typedef struct { char *ptr; int64_t len, cap; } String;
typedef struct { String name; int64_t id; int64_t hp; } Player;
typedef struct { Player **ptr; int64_t len, cap; } ArrPlayer;
typedef struct { ArrPlayer players; ArrInt log; int64_t damage, respawn_hp, deaths, total_hp, frame; } Game;

static void game_note(Game *THIS g, int64_t who) { push_int(&g->log, who); }

static NOINLINE void game_tick(Game *THIS g) {
    g->log.len = 0;
    Player **ps = g->players.ptr;
    int64_t n = g->players.len;
    for (int64_t i = 0; i < n; i++) {
        Player *p = ps[i];
        p->hp -= g->damage;
        if (p->hp <= 0) {
            g->deaths += 1;
            p->hp = g->respawn_hp;
            game_note(g, p->id);
        }
        g->total_hp += p->hp;
    }
    g->frame += 1;
}

int main(void) {
    h_start();
    Game *g = (Game *)launder(calloc(1, sizeof(Game)));
    g->damage = 1; g->respawn_hp = 1000;
    Player *block = (Player *)calloc(PLAYERS, sizeof(Player));
    g->players.ptr = (Player **)malloc(PLAYERS * sizeof(Player *));
    g->players.len = g->players.cap = PLAYERS;
    h_seed(17);
    for (int64_t i = 0; i < PLAYERS; i++) {
        Player *p = &block[i];
        p->id = i; p->hp = (int64_t)(h_rand() % 1000) + 1;
        g->players.ptr[i] = p;
    }
    uint64_t logged = 0;
    for (int f = 0; f < FRAMES; f++) {
        game_tick(g);
        for (int64_t k = 0; k < g->log.len; k++) logged += (uint64_t)g->log.ptr[k];
    }
    h_report(NAME, (uint64_t)g->total_hp + (uint64_t)g->deaths * 1000003u + logged + (uint64_t)g->frame);
    return 0;
}
