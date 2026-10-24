/* narrow_weak.c: rule 11 (narrowing) of the combined model meets its `weak` (a test on read).
   Report: tasks/96/redesign/phase3b-combined-attack.md, problem 1.

   The Ember program (the combined model's syntax):

       class Player:
           hp: int = 100
           inventory: Array[int] = []
       class Camera:
           target: weak Player = None

       fn main():
           players = [Player()]
           players[0].inventory.push(5)
           cam = Camera()
           cam.target = players[0]
           if cam.target is not None:          # rule 11: cam.target is a Player in this block
               players.pop()                   # the only strong handle goes: the Player dies
               cam.target.inventory.push(1)    # as written: still narrowed, so no test is emitted

   Rule 11 ends the narrowing at "any write of that place in any object (hidden writes included)".
   Under registration (the graphs group's `weak`), the Player's death wrote None into cam.target: a
   hidden write, so the narrowing ended. Under the combined model's `weak` (4.3), a death writes
   nothing into the weak field; the read tests the DEAD bit instead. So nothing ends the narrowing,
   and the second read is emitted as a plain load of a dead object whose fields were dropped.

   Built two ways (AddressSanitizer, as the graphs attacks did):
     clang -O1 -g -fsanitize=address narrow_weak.c -o narrow_weak_as_written.exe
     clang -O1 -g -fsanitize=address -DSAVED narrow_weak.c -o narrow_weak_fixed.exe
   SAVED: a statement that may drop a Player counts as a write of every `weak Player` place, so the
   narrowing ends at players.pop(); the program must test again (or hold `t = cam.target` before the
   pop, which keeps the Player alive), and the emitted read tests DEAD and finds None. */
#include "../../redesign/bench/harness.h"
#include <string.h>

#define SIDE (1u << 29)
#define DEAD (1u << 28)
#define LOW (DEAD - 1u)

typedef struct { int64_t *ptr; size_t len, cap; } IVec;
typedef struct Player { uint32_t rc; int32_t pad; int64_t hp; IVec inventory; } Player;  /* 4-byte header */
typedef struct Camera { uint32_t rc; Player *target; } Camera;                             /* weak field */

static void ivec_push(IVec *v, int64_t x) {
    if (v->len == v->cap) { v->cap = v->cap ? v->cap * 2 : 4; v->ptr = realloc(v->ptr, v->cap * sizeof *v->ptr); }
    v->ptr[v->len++] = x;
}

/* side records {strong, weak}, as combined/w3.c */
typedef struct { uint64_t strong, weak; } SideRec;
static SideRec side[16]; static uint32_t side_len;
static void weak_retain(Player *t) {
    if (!(t->rc & SIDE)) { uint32_t i = side_len++; side[i].strong = t->rc & LOW; side[i].weak = 0; t->rc = (t->rc & ~LOW) | SIDE | i; }
    side[t->rc & LOW].weak++;
}
static Player *weak_read(Player *t) { return (t && !(t->rc & DEAD)) ? t : NULL; }
static void drop_fields(Player *p) { free(p->inventory.ptr); /* the field is dropped; its words are not cleared */ }
static void release(Player *p) {                 /* the slow path taken: the count is in a side record */
    uint32_t i = p->rc & LOW;
    if (--side[i].strong != 0) return;
    p->rc |= DEAD;                               /* weak reads now give None; the block waits for its weak count */
    drop_fields(p);
}

int main(void) {
    h_start();
    Player *players[1]; size_t nplayers = 0;
    Player *p = calloc(1, sizeof(Player)); p->rc = 1; p->hp = 100;
    players[nplayers++] = p;
    ivec_push(&players[0]->inventory, 5);
    Camera *cam = calloc(1, sizeof(Camera)); cam->rc = 1;
    weak_retain(players[0]); cam->target = players[0];               /* cam.target = players[0] */

    if (weak_read(cam->target) != NULL) {                            /* if cam.target is not None: */
        Player *gone = players[--nplayers]; release(gone);           /*     players.pop()          */
#ifndef SAVED
        Player *t = cam->target;                                     /* narrowed: a plain load     */
        ivec_push(&t->inventory, 1);                                 /*     cam.target.inventory.push(1) */
        printf("as written: pushed into a dead Player's inventory (len %zu)\n", t->inventory.len);
#else
        Player *t = weak_read(cam->target);                          /* narrowing ended: tested again */
        if (t) ivec_push(&t->inventory, 1);
        printf("fixed: cam.target is %s after the pop\n", t ? "a Player" : "None");
#endif
    }
    return 0;
}
