/* model_tree.c: workload "tree" (M2), the cost row "one add and one subtract of the target's weak
   count, and before each add one compare with its maximum | every `weakref` link made, given another
   target or ended (R11), parent links included" (rules.md section 2; spec [WK-11], [OBJ-3], [RC-4]).

   The program, in spec 0.9.15_Hardened_1's syntax. It is run 3's W2 build (target/redesign3/
   25-c-writer-fix/w2.em) turned into building and rebuilding: each round builds a 1,000,000-node tree
   with parent links, moves a quarter of the nodes to new parents, reads every node once, and ends the
   whole tree.

       const N: int = 1_000_000
       const ROUNDS: int = 4

       class Node:
           value: i32
           parent: weakref Node = None         # a parent link (R11)
           children: Array[Node] = []

       fn main():
           checksum = 0
           for round in 0..ROUNDS:
               root = Node(value=0)
               nodes: Array[weakref Node] = [weakref root]     # every node by number, as links
               for i in 1..N:
                   n = Node(value=(i % 1000) as i32)
                   nodes.push(weakref n)
                   if nodes[rand() % i] exists as p:
                       n.parent = weakref p                    # the parent link: +1 on p
                       p.children.push(n)                      # n handed over to p
               for i in 1..N:
                   if (i + round) % 4 == 0:
                       if nodes[i] exists as node:
                           if node.parent exists as old:       # [WK-17]: proved, no header test
                               k = old.children.index_of(node).unwrap()
                               moved = old.children.swap_remove(k)
                               if nodes[rand() % i] exists as np:
                                   moved.parent = weakref np   # another target: +1 on np, -1 on old
                                   np.children.push(moved)
               for w in nodes:
                   if w exists as n:
                       checksum += n.value as int * len(n.children)
                       if n.parent exists as p:                # [WK-17]: proved
                           checksum += p.value as int
               # the round's locals end in reverse order: `nodes` (N links: -1 each), then `root`
               # (the whole tree: each child's parent link -1, each node's owners' share -1)
           println(checksum)

   [WK-17] holds here: both lines that give `parent` a target are followed by a push into a place
   the target owns alone; the one line that takes a node out (`swap_remove`) is followed, before the
   link is read, by the line that gives it another target; no children list is handed over whole;
   one owner each; no `drop` method; one package. Condition 2: no line between a view of a node and
   its parent read can end a Node. So every parent read is a load and a test for no link.

   What runs, per round (rule as written; every item is removed by -DPLAIN):
     weak counts  N index links made (+1, compare) and ended (-1); N-1 parent links made (+1, compare)
                  and ended with their child (-1); N/4 parent links given another target (+1 with
                  compare, -1); N owners' shares given back (-1, block freed at 0)
     header tests N-1 opens of `nodes[r]` in the build, about N/2 in the moves, N in the read pass
     language    bounds on `nodes[...]`, the floor `%` of a call's result, `unwrap`, overflow on `+`/`*`
   Build modes:
     (default)            model_tree: the rule as written
     -DPARENT_UNCOUNTED   salvage_tree_parentfree.c: parent links hold no weak count (run 3's `set by`
                          cost: uncounted parent links; the index links stay counted)
     -DPLAIN              plain_same_tree: this layout and this drop order with none of the above */
#include "m2_rt.h"

#define N 1000000
#define ROUNDS 4

static void node_end(Node *n) { node_drop(n); }

int main(void) {
    int64_t checksum = 0, round, i, j;
    h_start();
    h_seed(42);                                         /* rand() is h_rand() */
    for (round = 0; round < ROUNDS; round++) {
        Node *root = node_new(0);
        NodeArr nodes = { NULL, 0, 0 };
        weak_add(root); PUSH(&nodes, root);              /* [weakref root]: +1 [WK-11] */
        for (i = 1; i < N; i++) {
            Node *n = node_new((int32_t)(i % 1000));    /* i >= 1: C's % */
            Node *p;
            int64_t r;
            weak_add(n); PUSH(&nodes, n);                /* nodes.push(weakref n): +1 [WK-11] */
            r = fmod_i64((int64_t)h_rand(), i);          /* a call's result: floor % [TYP-28] */
            BOUNDS(r, nodes.len);
            p = open_link(nodes.ptr[r]);                 /* if nodes[...] exists as p: header test [WK-12] */
            if (p) {
                PARENT_LINK_ADD(p); n->parent = p;       /* n.parent = weakref p: +1 (it held None) [WK-11] */
                PUSH(&p->children, n);                   /* p.children.push(n) */
            } else {
                node_end(n);                             /* n was not handed over (never in this workload) */
            }
        }
        for (i = 1; i < N; i++) {
            Node *node, *old, *moved, *np;
            int64_t k, r;
            if ((i + round) % 4 != 0) continue;          /* both >= 0: C's % */
            BOUNDS(i, nodes.len);
            node = open_link(nodes.ptr[i]);              /* header test [WK-12] */
            if (!node) continue;
            old = PARENT_READ(node);                     /* [WK-17]: load + test for no link */
            if (!old) continue;
            k = index_of_unwrap(&old->children, node);
            moved = swap_remove(&old->children, k);      /* handed out; its link still names `old` */
            r = fmod_i64((int64_t)h_rand(), i);
            BOUNDS(r, nodes.len);
            np = open_link(nodes.ptr[r]);                /* header test [WK-12] */
            if (np) {
                PARENT_LINK_ADD(np);                     /* moved.parent = weakref np: +1 on np, compared */
                PARENT_LINK_SUB(moved->parent);          /*   -1 on the link's old target [WK-11] */
                moved->parent = np;
                PUSH(&np->children, moved);
            } else {
                node_end(moved);                         /* never in this workload */
            }
        }
        for (j = 0; j < nodes.len; j++) {
            Node *n = open_link(nodes.ptr[j]);           /* for w in nodes: if w exists as n: [WK-12] */
            Node *p;
            if (!n) continue;
            checksum = add_i64(checksum, mul_i64((int64_t)n->value, n->children.len));
            p = PARENT_READ(n);                          /* [WK-17]: proved */
            if (p) checksum = add_i64(checksum, (int64_t)p->value);
        }
        /* end of the round: `nodes` ends (each link -1), then `root` ends (the whole tree) */
#ifndef PLAIN
        for (j = 0; j < nodes.len; j++)
            if (nodes.ptr[j]) weak_sub(nodes.ptr[j]);
#endif
        free(nodes.ptr);
        node_end(root);
    }
#if defined(PLAIN)
    h_report("plain-same-tree", (uint64_t)checksum);
#elif defined(PARENT_UNCOUNTED)
    h_report("salvage-tree-parentfree", (uint64_t)checksum);
#else
    h_report("model-tree", (uint64_t)checksum);
#endif
    return 0;
}
