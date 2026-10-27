/* model_w2v.c: workload "w2v" (M2), the cost row "one test of the node's header and one move of the
   link's weak count (a subtract and an add), per step | a walk up a tree across a call that may change
   nodes (R11)" (rules.md section 2; spec [WK-18], [WK-17], [WK-12], [WK-11]).

   It is run 3's W2 (target/redesign3/25-c-writer-fix/w2.c; plain twin target/redesign2/bench/w2.c,
   checksum 647034814) with one change: every step of the walk calls a stored function value
   (`on_visit(p)`), which counts the step. The walk's own `depth += 1` moves into the visitor, so the
   checksum is W2's: 647034814 for every program of this workload.

       const N: int = 1_000_000

       class Node:
           value: i32
           parent: weakref Node = None
           children: Array[Node] = []

       class Counter:
           n: int = 0

       type Visit = fn(Node)

       fn prune(p: Node):                       # takes p out of its parent's list: p ends
           if p.parent exists as q:
               q.children.swap_remove(q.children.index_of(p).unwrap())

       fn main():
           root = Node(value=0)
           nodes: Array[weakref Node] = [weakref root]
           for i in 1..N:                       # W2's build, with the program's own parent links
               n = Node(value=(i % 1000) as i32)
               nodes.push(weakref n)
               if nodes[rand() % i] exists as p:
                   n.parent = weakref p
                   p.children.push(n)
           visits = shares mut Counter()
           on_visit: Visit = fn(p) shares mut visits: if p.value >= 0: visits.n += 1
           if len(args()) > 1:
               on_visit = prune                 # never given in the timed runs (runs.txt)
           for pass in 0..10:
               for leaf in nodes:               # [WK-18]'s walk, as E3021's message writes it
                   up = weakref leaf
                   while up exists as n:
                       if n.parent exists as p:
                           up = weakref p
                           on_visit(p)          # may end p (prune), so no view lives across it
                       else:
                           break
               for i in 1..N:                   # W2's re-parent, 1% of the nodes per pass
                   if (i + pass) % 100 == 0:
                       if nodes[i] exists as node:
                           if node.parent exists as old:
                               k = old.children.index_of(node).unwrap()
                               moved = old.children.swap_remove(k)
                               if nodes[rand() % i] exists as np:
                                   moved.parent = weakref np
                                   np.children.push(moved)
           checksum = visits.n
           for w in nodes:
               if w exists as n:
                   checksum += n.value as int * len(n.children)
           println(checksum)

   [WK-17] holds (as in model_tree.c; `prune` takes a node out and ends it, it never reads its link
   again), so `n.parent exists as p` is a load and a test for no link. Because `prune` can be stored
   in `on_visit`, the call may end a Node: no view of a node lives across it (R8 (g), [RC-5]) and the
   walk must hold a `weakref` link ([WK-18]). Which C runs for it is the form (one per build):

     FORM_MOVE (default)  the rule as written for a class that cannot use a region: per step one
                          header test (`while up exists as n`) and one move of the weak count
                          (`up = weakref p`: +1 on p with its compare, -1 on n); the first link +1 and
                          the last link's -1 once per walk. model_w2v_move.
     FORM_MOVE+DROPCLASS  `class Node` has `fn drop(this): DROPS += 1`: [WK-17] condition 1 fails, so
                          the parent read also tests p's header, every step; no region is allowed.
                          model_w2v_move_drop.
     FORM_REGION          the class has no drop: the walk runs in a deferred-drop region ([WK-18]):
                          one add and one subtract of a per-thread counter per walk; the first link
                          keeps its count; every later link is given only a live `p` inside the
                          region and holds no count; per step only the header test. model_w2v_region.
     FORM_VIEWS           the program never stores `prune` (every function stored in `on_visit` only
                          reads its node): the walk keeps views (`cur = ref n`, `while cur.parent
                          exists as p`, `cur = ref p`), nothing per step. model_w2v_views.
     PLAIN                plain C in this layout (the views walk with no checks). plain_same_w2v.
     NOVISIT (with FORM_VIEWS or PLAIN)  the bridge to run 3: no call, `depth += 1` in the walk.
   Salvage forms (each salvage_w2v_*.c defines one and includes this file): FORM_FREEONLY,
   FORM_DISPATCH, FORM_HOIST; the report gives their safety arguments.

   The function value is two words, its code and its state (R12), read from their slot at each call,
   since a call may store another function there; the slot is `volatile` so that neither C compiler
   devirtualises or inlines the call (the same slot in plain_w2v.c). Language checks kept, as run 3's
   model kept them: bounds on `nodes[...]`, the floor `%` of a call's result, `unwrap`, overflow on
   `+=`/`*` (the visitor's `visits.n += 1` included). Not emitted: the drops at the end of `main`, which
   run after `println` (run 3's model did the same). */
#include "m2_rt.h"

#define N 1000000
#define PASSES 10

#if !defined(FORM_MOVE) && !defined(FORM_REGION) && !defined(FORM_VIEWS) && !defined(FORM_FREEONLY) \
    && !defined(FORM_DISPATCH) && !defined(FORM_HOIST)
#ifdef PLAIN
#define FORM_VIEWS
#else
#define FORM_MOVE
#endif
#endif
#if defined(DROPCLASS) && (defined(FORM_REGION) || defined(FORM_HOIST))
#error "a deferred-drop region needs a class with no drop method of its own ([WK-18])"
#endif
#if defined(FORM_FREEONLY) && !defined(DROPCLASS)
#error "the deferred-free salvage is measured on the class with a drop method (DROPCLASS)"
#endif
#if defined(NOVISIT) && !defined(FORM_VIEWS)
#error "NOVISIT is the bridge to run 3's W2: views walk only"
#endif
#if defined(FORM_REGION) || defined(FORM_HOIST) || defined(FORM_FREEONLY)
#define HAS_REGION 1
#endif

/* ---- the deferred-drop region ([WK-18]; section 0's term), per thread ---- */
#ifdef HAS_REGION
static THREAD_LOCAL int64_t tls_region;            /* how many regions this thread is inside */
static THREAD_LOCAL NodeArr tls_aside;             /* objects (or, FREEONLY, blocks) set aside */
#endif

#ifdef FORM_FREEONLY
/* Salvage "deferred free": the drop runs at once (its visible actions are not moved); only the block's
   free waits for the region's end. */
#undef BLOCK_FREE
#define BLOCK_FREE(n) do { if (tls_region) PUSH(&tls_aside, (n)); else slab_free(&node_slab, (n)); } while (0)
#endif

#if defined(FORM_REGION) || defined(FORM_HOIST)
static void mark_ended(Node *n) {                  /* it and everything it owns, ended at once */
    int64_t k;
    n->count = 0;
    for (k = 0; k < n->children.len; k++) mark_ended(n->children.ptr[k]);
}
#endif

static void node_end(Node *n) {                    /* the last owner lets `n` go */
#if defined(FORM_REGION) || defined(FORM_HOIST)
    if (tls_region) { mark_ended(n); PUSH(&tls_aside, n); return; }   /* set aside whole [WK-18] */
#endif
    node_drop(n);
}

#ifdef HAS_REGION
static NOINLINE void region_end_work(void) {       /* the region ends: drop (or free) what it set aside */
    while (tls_aside.len) {
        Node *x = tls_aside.ptr[--tls_aside.len];
#ifdef FORM_FREEONLY
        slab_free(&node_slab, x);
#else
        node_drop(x);
#endif
    }
}
#define REGION_ENTER() (tls_region++)
#define REGION_LEAVE() do { if (--tls_region == 0 && tls_aside.len != 0) region_end_work(); } while (0)
#endif

/* ---- the function value `on_visit`: two words, read at each call (R12) ---- */
typedef void (*VisitCode)(void *state, Node *p);
static VisitCode volatile g_visit_code;
static void *volatile g_visit_state;
#define VISIT(p) g_visit_code(g_visit_state, (p))

typedef struct { uint32_t count, type; int64_t n; } Counter;   /* a `shares mut` Counter, its header */

/* fn(p) shares mut visits: if p.value >= 0: visits.n += 1 */
static void count_visit(void *state, Node *p) {
    Counter *c = (Counter *)state;
    if (p->value >= 0) c->n = add_i64(c->n, 1);   /* [TYP-8] */
}

#if !defined(FORM_VIEWS)
/* fn prune(p: Node): takes p out of its parent's list, and p (with everything under it) ends */
static void prune_visit(void *state, Node *p) {
    Node *q, *moved;
    (void)state;
    q = PARENT_READ(p);                             /* if p.parent exists as q: [WK-17] */
    if (!q) return;
    moved = swap_remove(&q->children, index_of_unwrap(&q->children, p));
    node_end(moved);                                /* the removed node is not kept: it ends here */
}
#endif

/* ---- the walks: one walk from `leaf` (an element of `nodes`, a counted link) ---- */

#if defined(FORM_MOVE) || defined(FORM_DISPATCH)
/* The rule as written ([WK-18]): per step one header test and one move of the weak count. */
static inline void walk_move(Node *leaf) {
    Node *up = leaf;                                /* up = weakref leaf */
    if (up) weak_add(up);                           /*   +1, compared with its maximum [WK-11] */
    while (up && up->count != 0) {                  /* while up exists as n: header test [WK-12] */
        Node *n = up;
        Node *p = PARENT_READ(n);                   /* if n.parent exists as p: [WK-17] load + test
                                                       for no link (DROPCLASS: + header test) */
        if (!p) break;                              /* else: break */
        weak_add(p);                                /* up = weakref p: +1 on p, compared [WK-11] */
        weak_sub(n);                                /*   and -1 on the link's old target */
        up = p;
        VISIT(p);                                   /* on_visit(p): may end p */
    }
    if (up) weak_sub(up);                           /* up ends: -1 [WK-11] */
}
#endif

#if defined(FORM_VIEWS) || defined(FORM_DISPATCH)
/* The visit changes no node: views, nothing per step ([WK-18] last sentence, [WK-17]). */
#ifdef NOVISIT
static int64_t g_depth;                             /* the bridge's depth counter */
#endif
static inline void walk_views(Node *leaf) {
    Node *cur = open_link(leaf), *p;                /* if leaf exists as n: header test; cur = ref n */
    if (!cur) return;
    while ((p = PARENT_READ(cur)) != NULL) {        /* while cur.parent exists as p: proved [WK-17] */
#ifdef NOVISIT
        g_depth = add_i64(g_depth, 1);              /* run 3's W2: depth += 1 */
#else
        VISIT(p);
#endif
        cur = p;                                    /* cur = ref p */
    }
}
#endif

#if defined(FORM_REGION) || defined(FORM_FREEONLY)
/* The walk in a deferred-drop region ([WK-18]) or, FREEONLY, a deferred-free region (salvage):
   one region per walk; the first link keeps its count; every later link is given a live `p` inside
   the region and holds none; per step the header test (FREEONLY with DROPCLASS: two). The first step
   is peeled: it is the one that gives the counted first link back. */
static inline void walk_region(Node *leaf) {
    Node *up = leaf, *p;                            /* up = weakref leaf */
    REGION_ENTER();
    if (up) {
        weak_add(up);                               /*   the first link: +1, compared [WK-11] */
        if (up->count != 0) {                       /* while up exists as n: (first step) */
            p = PARENT_READ(up);
            if (p) {
                weak_sub(up);                       /* up = weakref p: the first link's -1; p lives */
                up = p;                             /*   now, inside the region: no count */
                VISIT(p);
                while (up->count != 0) {            /* while up exists as n: header test, per step */
                    p = PARENT_READ(up);
                    if (!p) break;
                    up = p;                         /* up = weakref p: no count */
                    VISIT(p);
                }
                goto done;                          /* an uncounted link ends with nothing to do */
            }
        }
        weak_sub(up);                               /* the first link ends: -1 */
    }
done:
    REGION_LEAVE();
}
#endif

#ifdef FORM_HOIST
/* Salvage "hoist": the region covers the whole pass (entered once, in main), and the walk opens the
   leaf first (`if leaf exists as n: up = weakref n`), so the first link is also given a live node
   inside the region and holds no count: per walk nothing, per step the header test. */
static inline void walk_hoist(Node *leaf) {
    Node *up = open_link(leaf), *p;                 /* if leaf exists as n: header test; up = weakref n */
    if (!up) return;
    do {
        p = PARENT_READ(up);                        /* if n.parent exists as p: */
        if (!p) break;
        up = p;                                     /* up = weakref p: no count */
        VISIT(p);
    } while (up->count != 0);                       /* while up exists as n: header test, per step */
}
#endif

#ifdef FORM_DISPATCH
/* Salvage "dispatch": version the walk on the function actually stored. Every function the program
   may store in `on_visit` is known (R12); those whose summary changes no node are `count_visit` only.
   No stored function writes `on_visit`, so the code word read at the walk's start is the one every
   step calls. One load and one compare per walk; the rule's walk runs whenever `prune` is stored. */
static inline void walk_dispatch(Node *leaf) {
    if (g_visit_code == count_visit) walk_views(leaf);
    else walk_move(leaf);
}
#endif

int main(int argc, char **argv) {
    static Counter visits = { 1, 2, 0 };            /* visits = shares mut Counter() */
    NodeArr nodes = { NULL, 0, 0 };
    Node *root;
    int64_t i, j, pass, checksum;
    (void)argv;
    h_start();
    h_seed(42);
    root = node_new(0);
    weak_add(root); PUSH(&nodes, root);              /* nodes = [weakref root] */
    for (i = 1; i < N; i++) {
        Node *n = node_new((int32_t)(i % 1000)), *p;
        int64_t r;
        weak_add(n); PUSH(&nodes, n);                /* nodes.push(weakref n): +1 [WK-11] */
        r = fmod_i64((int64_t)h_rand(), i);
        BOUNDS(r, nodes.len);
        p = open_link(nodes.ptr[r]);                 /* header test [WK-12] */
        if (p) { PARENT_LINK_ADD(p); n->parent = p; PUSH(&p->children, n); }   /* n.parent = weakref p */
        else node_end(n);
    }
    g_visit_code = count_visit; g_visit_state = &visits;          /* on_visit = fn(p) shares mut visits: ... */
#if !defined(FORM_VIEWS)
    if (argc > 1) { g_visit_code = prune_visit; g_visit_state = NULL; }   /* on_visit = prune (never timed) */
#else
    (void)argc;
#endif
    for (pass = 0; pass < PASSES; pass++) {
#ifdef FORM_HOIST
        REGION_ENTER();
#endif
        for (j = 0; j < nodes.len; j++) {            /* for leaf in nodes: */
#if defined(FORM_MOVE)
            walk_move(nodes.ptr[j]);
#elif defined(FORM_VIEWS)
            walk_views(nodes.ptr[j]);
#elif defined(FORM_REGION) || defined(FORM_FREEONLY)
            walk_region(nodes.ptr[j]);
#elif defined(FORM_HOIST)
            walk_hoist(nodes.ptr[j]);
#elif defined(FORM_DISPATCH)
            walk_dispatch(nodes.ptr[j]);
#endif
        }
#ifdef FORM_HOIST
        REGION_LEAVE();
#endif
        for (i = 1; i < N; i++) {                    /* W2's re-parent */
            Node *node, *old, *moved, *np;
            int64_t k, r;
            if ((i + pass) % 100 != 0) continue;
            BOUNDS(i, nodes.len);
            node = open_link(nodes.ptr[i]);
            if (!node) continue;
            old = PARENT_READ(node);
            if (!old) continue;
            k = index_of_unwrap(&old->children, node);
            moved = swap_remove(&old->children, k);
            r = fmod_i64((int64_t)h_rand(), i);
            BOUNDS(r, nodes.len);
            np = open_link(nodes.ptr[r]);
            if (np) {
                PARENT_LINK_ADD(np);                 /* moved.parent = weakref np: +1, compared */
                PARENT_LINK_SUB(moved->parent);      /*   -1 on the old target */
                moved->parent = np;
                PUSH(&np->children, moved);
            } else {
                node_end(moved);
            }
        }
    }
#ifdef NOVISIT
    checksum = g_depth;
#else
    checksum = visits.n;                             /* checksum = visits.n */
#endif
    for (j = 0; j < nodes.len; j++) {
        Node *n = open_link(nodes.ptr[j]);
        if (n) checksum = add_i64(checksum, mul_i64((int64_t)n->value, n->children.len));
    }
#if defined(PLAIN) && defined(NOVISIT)
    h_report("plain-same-w2v-novisit", (uint64_t)checksum);
#elif defined(PLAIN)
    h_report("plain-same-w2v", (uint64_t)checksum);
#elif defined(NOVISIT)
    h_report("model-w2v-views-novisit", (uint64_t)checksum);
#elif defined(FORM_MOVE) && defined(DROPCLASS)
    h_report("model-w2v-move-drop", (uint64_t)checksum);
#elif defined(FORM_MOVE)
    h_report("model-w2v-move", (uint64_t)checksum);
#elif defined(FORM_REGION)
    h_report("model-w2v-region", (uint64_t)checksum);
#elif defined(FORM_VIEWS)
    h_report("model-w2v-views", (uint64_t)checksum);
#elif defined(FORM_FREEONLY)
    h_report("salvage-w2v-freeonly", (uint64_t)checksum);
#elif defined(FORM_DISPATCH)
    h_report("salvage-w2v-dispatch", (uint64_t)checksum);
#elif defined(FORM_HOIST)
    h_report("salvage-w2v-hoist", (uint64_t)checksum);
#endif
    return 0;
}
