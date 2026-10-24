/* backlink_filter.c: the graphs group's back links (a push into `children` first takes the node out of
   its old holder's list, its count moving with it) meet the containers group's loop rule (a loop whose
   body may change the list it walks is a one-pass filter; "a removed element is dropped at the end of
   its step"). Problem 6 of tasks/96/redesign/phase3b-combined-attack.md.

   The Ember program (the combined model's syntax):

       class Node:
           value: int
           parent: Option[Node] = backlink(children)
           children: Array[Node] = []

       old = Node(value=0); new = Node(value=1)
       for v in [3, 7, 9, 2]: old.children.push(Node(value=v))
       for c in old.children:            # the body may change old.children (push's hidden removal):
           if c.value > 5:               #   the loop is the one-pass filter
               new.children.push(c)      # c leaves old.children (a hidden removal of the walked list)
       println([c.value for c in new.children], [c.value for c in old.children])   # [7, 9] [3, 2]

   The hidden removal takes the CURRENT element out of the walked list, so the filter must map it
   (rule 7: "other removals in the body are mapped"). Two ways to map it:
     -DAS_REMOVE  as `old.children.remove(c)`: the filter releases the element at the end of its step;
                  but the push moved the handle into new.children without a new count, so the count is
                  now one too low and the node is freed while new.children holds it
     default      as a move out of the walked list: the filter closes the gap and releases nothing
                  (the count moved with the handle)
     clang -O1 -g -fsanitize=address [-DAS_REMOVE] backlink_filter.c -o backlink_filter[_as_remove].exe */
#include "../../redesign/bench/harness.h"
#include <string.h>

typedef struct Node Node;
typedef struct { Node **ptr; size_t len, cap; } NVec;
struct Node { uint32_t rc; int32_t value; Node *parent; NVec children; };

static void push_raw(NVec *v, Node *x) { if (v->len == v->cap) { v->cap = v->cap ? v->cap * 2 : 4; v->ptr = realloc(v->ptr, v->cap * sizeof(Node *)); } v->ptr[v->len++] = x; }
static Node *node_new(int32_t value) { Node *n = calloc(1, sizeof *n); n->rc = 1; n->value = value; return n; }
static void release(Node *n) {
    if (--n->rc != 0) return;
    for (size_t k = 0; k < n->children.len; k++) { n->children.ptr[k]->parent = NULL; release(n->children.ptr[k]); }
    free(n->children.ptr); free(n);
}

/* The walk of old.children as the one-pass filter: positions r (read) and w (write). */
typedef struct { NVec *list; size_t r, w; int current_gone; } Filter;

/* holder.children.push(c) with c.parent kept by the language, while `f` walks c's old holder's list */
static void children_push_in_filter(Node *holder, Node *c, Filter *f) {
    Node *old = c->parent;
    if (old && &old->children == f->list && f->list->ptr[f->r] == c) {
        f->current_gone = 1;                 /* mapped onto the filter: the current element leaves */
    } else if (old) {
        size_t k = 0; while (old->children.ptr[k] != c) k++;   /* (not reached here) */
        memmove(&old->children.ptr[k], &old->children.ptr[k + 1], (old->children.len - k - 1) * sizeof(Node *));
        old->children.len--;
    }
    push_raw(&holder->children, c);          /* the handle moves: no count change */
    c->parent = holder;
}

int main(void) {
    h_start();
    Node *old = node_new(0), *new = node_new(1);
    int32_t vals[4] = {3, 7, 9, 2};
    for (int i = 0; i < 4; i++) { Node *n = node_new(vals[i]); push_raw(&old->children, n); n->parent = old; }
    Filter f = {&old->children, 0, 0, 0};
    for (f.r = 0; f.r < f.list->len; f.r++) {
        Node *c = f.list->ptr[f.r];
        f.current_gone = 0;
        if (c->value > 5) children_push_in_filter(new, c, &f);
        if (!f.current_gone) { f.list->ptr[f.w++] = c; continue; }
#ifdef AS_REMOVE
        release(c);                          /* "a removed element is dropped at the end of its step" */
#endif
    }
    f.list->len = f.w;
    printf("new.children:"); for (size_t k = 0; k < new->children.len; k++) printf(" %d", new->children.ptr[k]->value);
    printf("   old.children:"); for (size_t k = 0; k < old->children.len; k++) printf(" %d", old->children.ptr[k]->value);
    printf("\n");
    release(old); release(new);
    printf("both trees released\n");
    return 0;
}
