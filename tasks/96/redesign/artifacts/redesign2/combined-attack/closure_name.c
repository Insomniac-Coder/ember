/* closure_name.c: names (ownership group A11, containers group A2: `boss = enemies[0]` is "element 0
   of enemies", a position) meet captures (graphs group A12: a stored closure's captured variables live
   in a box). Problem 4 of tasks/96/redesign/phase3b-combined-attack.md. The containers group asked the
   closures group to decide what a stored closure captures when it captures a name
   (phase3b-containers.md section 7, item 6); neither the graphs reports nor the combined model answer.

   The Ember program (the combined model's syntax), Enemy a struct:

       struct Enemy:
           hp: int
       class Game:
           enemies: Array[Enemy] = []
           buttons: Array[fn()] = []

       fn setup(g: Game):
           boss = g.enemies[0]                    # a name: element 0 of g.enemies
           g.buttons.push(fn(): boss.hp -= 1)     # the closure escapes into g.buttons
       fn wave(g: Game):                          # another function, which knows nothing of the closure
           g.enemies.remove(0)                    # the boss is defeated and leaves the list
           for i in range(64): g.enemies.push(Enemy(hp=i))   # a new wave: the buffer moves

       g = Game(); g.enemies.push(Enemy(hp=10)); g.enemies.push(Enemy(hp=20))
       setup(g); wave(g)
       for b in g.buttons: b()
       println(g.enemies[0].hp)                   # Python (class Enemy): 20, the boss object took the hit

   Rule 10 picks, per captured variable, copy, move, pointer or box. A name is not a variable holding
   a value, so each reading of the rule is tried here:
     -DREADING=1  the box holds the element's address (a pointer capture): the wave's pushes move the
                  buffer, the click writes into freed memory
     -DREADING=2  the box holds the path (the Game handle and position 0): wave() cannot move a position
                  it does not know about, so the click hits whatever is at position 0 now (silently
                  wrong; with an empty list, a panic)
     -DREADING=3  the box holds a copy of the element (rule 10's "copy"): the click changes the copy;
                  nothing in the game changes, and Python's boss object is not what the program meant
   The fix (-DREADING=4, Enemy a class, Python's own program): an escaping closure that captures a
   name of a CLASS element captures the handle, retained; for a STRUCT element it is a compile-time
   error with fix-its (`copy(boss)` to say a snapshot is meant, make Enemy a class, or capture the
   list and an index). A closure that provably dies before the name's function returns keeps the path,
   as any other use of the name does.
     clang -O1 -g -fsanitize=address -DREADING=n closure_name.c -o closure_name_n.exe */
#include "../../redesign/bench/harness.h"
#include <string.h>
#ifndef READING
#define READING 1
#endif

#if READING == 4
typedef struct Enemy { uint32_t rc; int32_t pad; int64_t hp; } Enemy;   /* class Enemy: a handle */
typedef Enemy *Elem;
#define HP(e) ((e)->hp)
#else
typedef struct Enemy { int64_t hp; } Enemy;                              /* struct Enemy: inline */
typedef Enemy Elem;
#define HP(e) ((e).hp)
#endif
typedef struct { Elem *ptr; size_t len, cap; } EVec;
typedef struct Game Game;
typedef struct Box { uint32_t rc;
#if READING == 1
    Enemy *addr;                      /* the element's address */
#elif READING == 2
    Game *g; size_t pos;              /* the path: the game handle and the position */
#elif READING == 3
    Enemy copy;                       /* a copy of the element */
#else
    Enemy *boss;                      /* the handle, retained */
#endif
} Box;
struct Game { uint32_t rc; EVec enemies; Box *buttons[4]; int nbuttons; };

static void push(EVec *v, Elem x) { if (v->len == v->cap) { v->cap = v->cap ? v->cap * 2 : 4; v->ptr = realloc(v->ptr, v->cap * sizeof(Elem)); } v->ptr[v->len++] = x; }
static void remove_at(EVec *v, size_t k) {
    if (k >= v->len) { fprintf(stderr, "panic: index out of range\n"); exit(101); }
#if READING == 4
    Enemy *gone = v->ptr[k];
#endif
    memmove(&v->ptr[k], &v->ptr[k + 1], (v->len - k - 1) * sizeof(Elem)); v->len--;
#if READING == 4
    if (--gone->rc == 0) free(gone);  /* the list's handle is released; the closure's keeps it */
#endif
}
static Elem new_enemy(int64_t hp) {
#if READING == 4
    Enemy *e = calloc(1, sizeof *e); e->rc = 1; e->hp = hp; return e;
#else
    return (Enemy){hp};
#endif
}
static void click(Box *b) {           /* fn(): boss.hp -= 1 */
#if READING == 1
    b->addr->hp -= 1;
#elif READING == 2
    if (b->pos >= b->g->enemies.len) { fprintf(stderr, "panic: index out of range\n"); exit(101); }
    b->g->enemies.ptr[b->pos].hp -= 1;
#elif READING == 3
    b->copy.hp -= 1;
#else
    b->boss->hp -= 1;
#endif
}
static void setup(Game *g) {
    Box *b = calloc(1, sizeof *b); b->rc = 1;
#if READING == 1
    b->addr = &g->enemies.ptr[0];
#elif READING == 2
    b->g = g; b->pos = 0;
#elif READING == 3
    b->copy = g->enemies.ptr[0];
#else
    b->boss = g->enemies.ptr[0]; b->boss->rc++;     /* the captured handle: one more count */
#endif
    g->buttons[g->nbuttons++] = b;
}
static void wave(Game *g) {
    remove_at(&g->enemies, 0);
    for (int i = 0; i < 64; i++) push(&g->enemies, new_enemy(i));
}

int main(void) {
    h_start();
    Game *g = calloc(1, sizeof *g); g->rc = 1;
    push(&g->enemies, new_enemy(10)); push(&g->enemies, new_enemy(20));
    setup(g); wave(g);
    for (int i = 0; i < g->nbuttons; i++) click(g->buttons[i]);
    static const char *what[] = {"", "address", "path", "copy", "handle (class Enemy)"};
#if READING == 4
    printf("reading %d (%s): enemies[0].hp = %lld, the boss (out of the list) hp = %lld\n", READING, what[READING],
           (long long)HP(g->enemies.ptr[0]), (long long)g->buttons[0]->boss->hp);
#else
    printf("reading %d (%s): enemies[0].hp = %lld\n", READING, what[READING], (long long)HP(g->enemies.ptr[0]));
#endif
    return 0;
}
