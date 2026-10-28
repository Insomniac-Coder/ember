/* model_this_tick.c: workload this_tick, what Ember emits under the rule as written.

   The Ember program (AN-17, run4/v5/area-B.md line 190, with the log holding the player's number
   instead of a formatted text, so the loop's cold path is a push and not a text format):

       class Player:
           name: String
           id: int
           hp: int

       class Game:
           players: Array[Player]
           log: Array[int]
           damage: int = 1
           respawn_hp: int = 1000
           deaths: int = 0
           total_hp: int = 0
           frame: int = 0

           fn note(who: int):
               this.log.push(who)

           fn tick():
               this.log.clear()
               for ref mut p in this.players:
                   p.hp -= this.damage
                   if p.hp <= 0:
                       this.deaths += 1
                       p.hp = this.respawn_hp
                       this.note(p.id)
                   this.total_hp += p.hp
               this.frame += 1

   `this.note(p.id)` while `p` is used is accepted: `note`'s body changes only `log` (worked out,
   R5, [BRW-10]), so it cannot touch the player `p` names (R8 (f)).

   The rule this workload measures: `this` is never compiled with noalias (verdict 18; R4 last point,
   R5; [BRW-22], [EXC-15], [SIMD-3]): a method that changes its object may also reach it through a link
   (a turret aimed at itself), so `Game *` carries no `restrict`. -DNOALIAS_THIS (used only by
   salvage_this_tick_typeproof.c) adds it.

   Checks kept, each tagged: [TYP-8] overflow of every `+=` / `-=` on `int`. No bounds check: the walk
   is by element. The walk reads the list's pointer and length once ([CTL-2] / R8 (c): the list
   cannot change while a loop walks it, run 3's w1 model, AN-06 point 7). `log.clear()` on an
   `Array[int]` drops nothing: one store. */
#include "m4.h"

#ifdef NOALIAS_THIS
#define THIS RESTRICT
#ifndef NAME
#define NAME "model_this_tick_noalias"
#endif
#else
#define THIS
#define NAME "model_this_tick"
#endif

#define PLAYERS 16384
#define FRAMES 12000

typedef struct { char *ptr; int64_t len, cap; } String;
/* [OBJ-1] (0.9.15): a class object is its own block with an 8-byte header {count u32, type u32}. */
typedef struct { uint32_t count, type; String name; int64_t id; int64_t hp; } Player;
typedef struct { Player **ptr; int64_t len, cap; } ArrPlayer;
typedef struct { uint32_t count, type; ArrPlayer players; ArrInt log; int64_t damage, respawn_hp, deaths, total_hp, frame; } Game;

static void Game_note(Game *THIS this_, int64_t who) { push_int(&this_->log, who); }

static NOINLINE void Game_tick(Game *THIS this_) {
    this_->log.len = 0;                                         /* this.log.clear(): ints, nothing to drop */
    Player **ps = this_->players.ptr;                           /* R8 (c): the walked list's words, read once */
    int64_t n = this_->players.len;
    for (int64_t i = 0; i < n; i++) {
        Player *p = ps[i];                                      /* `ref mut p`: the element's address */
        p->hp = ck_sub(p->hp, this_->damage);                   /* [TYP-8] '-=' */
        if (p->hp <= 0) {
            this_->deaths = ck_add(this_->deaths, 1);           /* [TYP-8] '+=' */
            p->hp = this_->respawn_hp;
            Game_note(this_, p->id);
        }
        this_->total_hp = ck_add(this_->total_hp, p->hp);       /* [TYP-8] '+=' */
    }
    this_->frame = ck_add(this_->frame, 1);                     /* [TYP-8] '+=' */
}

int main(void) {
    h_start();
    Game *g = (Game *)launder(calloc(1, sizeof(Game)));
    g->count = 1; g->type = 2; g->damage = 1; g->respawn_hp = 1000;
    Player *block = (Player *)calloc(PLAYERS, sizeof(Player)); /* the slab: class objects carved in order */
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
        for (int64_t k = 0; k < g->log.len; k++) logged += (uint64_t)g->log.ptr[k];  /* the checksum's own read, as in plain C */
    }
    h_report(NAME, (uint64_t)g->total_hp + (uint64_t)g->deaths * 1000003u + logged + (uint64_t)g->frame);
    return 0;
}
