/* A2 attacks: what the answer's rules (phase3b-containers.md section 2.3 (b)) compile two ordinary
   Python programs into, against what Python prints (probes/py_attack_output.txt). Each "as written"
   function is the code the rules give; each "saved" function is the fixed rule. Plain data only, so
   nothing here is a memory error: the attacks are wrong answers with no error.

   1. The swap idiom.            Python and Ember source:
          tmp = xs[0]                 (rule 1: tmp names position 0, the element is not a number)
          xs[0] = xs[1]
          xs[1] = tmp
   2. An element passed to a method that changes its list.
          fn hit(mut self, mut e: Enemy):      def hit(self, e):
              e.hp -= 10                           e.hp -= 10
              self.purge()                         self.purge()
          w.hit(w.enemies[1])                  w.hit(w.enemies[1])
      purge removes the dead enemies (hp <= 0) in place, keeping the order. Rule 6: hit may change
      w.enemies, so the plain-data element is passed by copy and written back after the call. */
#include "../harness.h"   /* SetErrorMode: no error dialogs */
#include <string.h>
typedef struct { int hp; } Enemy;
typedef struct { Enemy data[8]; size_t len; } List;
static void show(const char *what, const List *l) {
    printf("%-46s [", what);
    for (size_t k = 0; k < l->len; k++) printf("%sE%d", k ? ", " : "", l->data[k].hp);
    printf("]\n");
}
static void purge(List *l) {                     /* remove dead enemies in place, order kept */
    size_t w = 0;
    for (size_t r = 0; r < l->len; r++) if (l->data[r].hp > 0) l->data[w++] = l->data[r];
    l->len = w;
}

int main(void) {
    h_start();
    { /* 1. swap, as written: tmp is the position 0 */
        List xs = {{{1}, {2}}, 2};
        size_t tmp = 0;                          /* tmp = xs[0] */
        xs.data[0] = xs.data[1];                 /* xs[0] = xs[1] */
        xs.data[1] = xs.data[tmp];               /* xs[1] = tmp: reads position 0, already replaced */
        show("1. swap, rules as written (Python: [E2, E1]):", &xs);
    }
    { /* 1. swap, saved: replacing the slot a name points at moves the old value into the name */
        List xs = {{{1}, {2}}, 2};
        size_t tmp = 0; int tmp_has_own = 0; Enemy tmp_own;
        if (tmp == 0) { tmp_own = xs.data[0]; tmp_has_own = 1; }   /* xs[0] is about to be replaced */
        xs.data[0] = xs.data[1];
        xs.data[1] = tmp_has_own ? tmp_own : xs.data[tmp];
        show("1. swap, saved:", &xs);
    }
    { /* 2. hit + purge, as written: copy in, call, write back to the old position */
        List en = {{{0}, {50}, {70}}, 3};
        size_t pos = 1; Enemy copy = en.data[pos];
        copy.hp -= 10; purge(&en);               /* the body of hit, on the copy */
        if (pos < en.len) en.data[pos] = copy;   /* write back: position 1 is now another enemy */
        show("2. hit + purge, rules as written (Python: [E40, E70]):", &en);
    }
    { /* 2. hit + purge, saved: the callee gets a place (list, position); its removals move the position */
        List en = {{{0}, {50}, {70}}, 3};
        size_t pos = 1;                          /* hidden in-out parameter of this copy of hit */
        en.data[pos].hp -= 10;
        {   /* purge, compiled for a caller holding a place in this list: j -= (k < j) per removal */
            size_t w = 0, newpos = pos;
            for (size_t r = 0; r < en.len; r++) {
                if (en.data[r].hp > 0) { if (r == pos) newpos = w; en.data[w++] = en.data[r]; }
            }
            en.len = w; pos = newpos;
        }
        show("2. hit + purge, saved:", &en);
    }
    return 0;
}
