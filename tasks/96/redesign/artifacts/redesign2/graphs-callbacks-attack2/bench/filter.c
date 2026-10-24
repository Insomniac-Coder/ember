/* filter.c: Python's filter idiom on a back-linked child list (A8). The Ember program (proposed):

       class Node:
           v: int
           parent: Option[Node] = backlink(children)
           children: Array[Node] = []

       for r in range(ROUNDS):
           dropped = [c for c in p.children if (c.v * 7 + r) % 3 == 0]
           p.children = [c for c in p.children if (c.v * 7 + r) % 3 != 0]   # assigning the whole field
           for c in dropped:
               p.children.push(c)                                          # put back at the end

     -DLITERAL  the report's rules read as written: "every operation that puts an object into g
                (... assigning the whole field) first takes the object out of its current holder's g
                (search, then the order-keeping remove)", then rule 3 for what is left of the old list.
                Each kept child's current holder is p itself, so each is searched for and removed from
                the old list one by one.
     -DORDERED  the save: one membership change: (1) unlink every element of the old value, (2) link
                every element of the new value (taking it out of another holder only if it still has
                one), (3) release the old value.
     -DORDERED -DNOCOUNT  the same with every count update left out (not safe; only to take the cost
                apart: what is left is the link's own work).
     -DPLAIN    plain C: build the kept array, clear the dropped children's parent, free the old array.
   Every node is also held by `all`, so nothing is freed. Overflow checks are left out. */
#include "ember_model.h"
#ifndef NCH
#define NCH 10000
#endif
#define ROUNDS 10

typedef struct Node { uint32_t rc, type; int64_t v; struct Node *parent; EVec children; } Node;
static Node *node_new(int64_t v) { Node *n = calloc(1, sizeof *n); n->rc = 1; n->type = 1; n->v = v; return n; }
#ifdef NOCOUNT
#define RC_INC(n) ((void)0)
static inline void node_release(Node *n) { (void)n; }
#else
#define RC_INC(n) ((n)->rc++)
static inline void node_release(Node *n) { if (--n->rc == 0) e_panic("not reached: `all` holds every node"); }
#endif

/* p.children.push(c), back-link rules 2 (c has no holder here) */
static void children_push(Node *p, Node *c) {
#ifndef PLAIN
    if (c->parent) {
        Node *old = c->parent;
        evec_remove_at(&old->children, evec_index_of(&old->children, c));   /* the count moves */
    } else RC_INC(c);
#endif
    evec_push(&p->children, c);
    c->parent = p;
}

int main(void) {
    h_start();
    EVec all = {0};
    Node *p = node_new(-1); evec_push(&all, p);
    for (int64_t i = 0; i < NCH; i++) { Node *c = node_new(i); evec_push(&all, c); children_push(p, c); }
    for (int r = 0; r < ROUNDS; r++) {
        EVec kept = {0}, dropped = {0};
        for (size_t i = 0; i < p->children.len; i++) {        /* the two comprehensions */
            Node *c = p->children.ptr[i];
#ifndef PLAIN
            RC_INC(c);
#endif
            if ((c->v * 7 + r) % 3 != 0) evec_push(&kept, c); else evec_push(&dropped, c);
        }
        EVec old = p->children;                               /* p.children = kept */
#if defined(LITERAL)
        for (size_t i = 0; i < kept.len; i++) {
            Node *c = kept.ptr[i];
            if (c->parent) {                                  /* take it out of its holder: p's old list */
                evec_remove_at(&c->parent->children, evec_index_of(&c->parent->children, c));
                node_release(c);                              /* the old list's count */
            }
            c->parent = p;
        }
        old = p->children;                                    /* what is left of the old list: rule 3 */
        for (size_t i = 0; i < old.len; i++) { Node *c = old.ptr[i]; c->parent = NULL; node_release(c); }
#elif defined(ORDERED)
        for (size_t i = 0; i < old.len; i++) ((Node *)old.ptr[i])->parent = NULL;           /* 1 */
        for (size_t i = 0; i < kept.len; i++) {                                               /* 2 */
            Node *c = kept.ptr[i];
            if (c->parent) { evec_remove_at(&c->parent->children, evec_index_of(&c->parent->children, c)); node_release(c); }
            c->parent = p;
        }
        for (size_t i = 0; i < old.len; i++) node_release(old.ptr[i]);                      /* 3 */
#else
        for (size_t i = 0; i < dropped.len; i++) ((Node *)dropped.ptr[i])->parent = NULL;
#endif
        free(old.ptr);
        p->children = kept;
        for (size_t i = 0; i < dropped.len; i++) {            /* for c in dropped: p.children.push(c) */
            children_push(p, dropped.ptr[i]);
#ifndef PLAIN
            node_release(dropped.ptr[i]);                     /* `dropped` dies */
#endif
        }
        free(dropped.ptr);
    }
    uint64_t sum = 0;
    for (size_t i = 0; i < p->children.len; i++) {
        Node *c = p->children.ptr[i];
        sum += (uint64_t)(c->v * (int64_t)(i + 1)) + (c->parent == p ? 1u : 1000000u);
    }
#define STR2(x) #x
#define STR(x) STR2(x)
#if defined(LITERAL)
    h_report("filter-literal-n" STR(NCH), sum);
#elif defined(ORDERED) && defined(NOCOUNT)
    h_report("filter-ordered-nocount-n" STR(NCH), sum);
#elif defined(ORDERED)
    h_report("filter-ordered-n" STR(NCH), sum);
#else
    h_report("filter-plain-n" STR(NCH), sum);
#endif
    return 0;
}
