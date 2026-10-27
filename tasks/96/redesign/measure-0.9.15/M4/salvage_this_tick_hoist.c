/* salvage_this_tick_hoist.c: the model with `this`'s fields held in locals across the walk.

   Idea: the C compiler cannot keep `this->damage` or `this->total_hp` in a register when a store
   it cannot place (here `p->hp`, and the out-of-line growth of `log`) might change them. Ember's
   front end can: it already works out every place each line and call reaches (R5, R8). So it reads
   each field the loop only reads into a local before the loop, keeps each field the loop changes in
   a local, and writes it back after the loop, and before any call in the loop that reads it.

   Safety, for this loop: the loop changes `p.hp` (a `Player`: another class, its own block, never
   part of a `Game`), `this.deaths`, `this.total_hp` (written back after the loop) and `this.log`
   (through `note`, whose body reads and changes only `log`, so no write-back is needed before it).
   Nothing in the loop writes `damage` or `respawn_hp` by any path. A panic ([PAN-1]) ends the process
   with no unwinding, so a field not yet written back is never seen. Where the proof fails (a write
   through a link of type `Game` in the loop, or a call that may read a cached field), the field is
   used in place, as in the model. This is [SIMD-3]'s per-loop alias fact for `this`, made by the
   front end instead of asked of the C compiler. Same checks as the model, same order of effects. */
#include "m4.h"

#define PLAYERS 16384
#define FRAMES 12000

typedef struct { char *ptr; int64_t len, cap; } String;
typedef struct { uint32_t count, type; String name; int64_t id; int64_t hp; } Player;     /* [OBJ-1] */
typedef struct { Player **ptr; int64_t len, cap; } ArrPlayer;
typedef struct { uint32_t count, type; ArrPlayer players; ArrInt log; int64_t damage, respawn_hp, deaths, total_hp, frame; } Game;

static void Game_note(Game *this_, int64_t who) { push_int(&this_->log, who); }

static NOINLINE void Game_tick(Game *this_) {
    this_->log.len = 0;
    Player **ps = this_->players.ptr;                           /* R8 (c) */
    int64_t n = this_->players.len;
    int64_t damage = this_->damage, respawn_hp = this_->respawn_hp;   /* only read in the loop (proved) */
    int64_t deaths = this_->deaths, total_hp = this_->total_hp;       /* changed only by the loop's own lines (proved) */
    for (int64_t i = 0; i < n; i++) {
        Player *p = ps[i];
        p->hp = ck_sub(p->hp, damage);                          /* [TYP-8] */
        if (p->hp <= 0) {
            deaths = ck_add(deaths, 1);                         /* [TYP-8] */
            p->hp = respawn_hp;
            Game_note(this_, p->id);                            /* reads/changes only `log`: no write-back needed */
        }
        total_hp = ck_add(total_hp, p->hp);                     /* [TYP-8] */
    }
    this_->deaths = deaths; this_->total_hp = total_hp;         /* written back after the loop */
    this_->frame = ck_add(this_->frame, 1);                     /* [TYP-8] */
}

int main(void) {
    h_start();
    Game *g = (Game *)launder(calloc(1, sizeof(Game)));
    g->count = 1; g->type = 2; g->damage = 1; g->respawn_hp = 1000;
    Player *block = (Player *)calloc(PLAYERS, sizeof(Player));
    g->players.ptr = (Player **)malloc(PLAYERS * sizeof(Player *));
    g->players.len = g->players.cap = PLAYERS;
    h_seed(17);
    for (int64_t i = 0; i < PLAYERS; i++) {
        Player *p = &block[i];
        p->count = 1; p->type = 1; p->id = i; p->hp = (int64_t)(h_rand() % 1000) + 1;
        g->players.ptr[i] = p;
    }
    uint64_t logged = 0;
    for (int f = 0; f < FRAMES; f++) {
        Game_tick(g);
        for (int64_t k = 0; k < g->log.len; k++) logged += (uint64_t)g->log.ptr[k];
    }
    h_report("salvage_this_tick_hoist", (uint64_t)g->total_hp + (uint64_t)g->deaths * 1000003u + logged + (uint64_t)g->frame);
    return 0;
}
