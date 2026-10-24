/* attacks.c: what the code the ownership answers say the compiler emits does on two programs those
 * answers' rules accept. Written by me; run under AddressSanitizer so a use of freed memory is reported.
 *
 *   attacks.exe 1   names follow values, rule 3 as written ("a removal that shifts elements, a clear,
 *                   or a store ... ends the name"): pop() shifts nothing, so the name `child` still
 *                   names root.children[child_at] after the pop, and its next use writes the dropped
 *                   node's freed buffer.
 *   attacks.exe 2   the same program with the fix: the name keeps the popped value (Python's meaning).
 *   attacks.exe 3   field effects with `mut` inferred from the body: std's CppVector.push_back writes no
 *                   field of `self`, so a span taken before it is kept across it; the push reallocates.
 *   attacks.exe 4   field effects, "detach before running drop code" required only for clear():
 *                   `w.items = []` emitted as today's compiler emits a handle-field store (release the
 *                   old value, then store), so each Item's drop walks the list being dropped.
 *   attacks.exe 5   the same with the fix for every reshape: take the old value out, store the new one,
 *                   then drop the old (Python's order).
 *   attacks.exe 6   A18 as written ("a helper may run on a half-built self if it reads only fields
 *                   already set, writes only those or the ones it sets"): the helper `load` is compiled
 *                   once, as a method of a finished Game, so its `self.assets = [...]` drops the old
 *                   value first, which on a half-built Game is whatever the stack held.
 *   attacks.exe 7   the fix: the call from init uses a second copy of `load` in which the stores to the
 *                   unset fields are first stores (no drop).
 *
 * Build: clang -O1 -g -fsanitize=address attacks.c -o attacks.exe   (clang 22.1.8)
 */
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct Node Node;
typedef struct { Node *p; size_t len, cap; } NodeVec;          /* ember_vec: pointer, length, capacity */
struct Node { int64_t value; NodeVec children; };

static void vec_push(NodeVec *v, Node n) {
    if (v->len == v->cap) {
        size_t cap = v->cap ? v->cap * 2 : 4;
        Node *p = (Node *)realloc(v->p, cap * sizeof(Node));
        if (!p) abort();
        v->p = p; v->cap = cap;
    }
    v->p[v->len++] = n;                                          /* a move: the bits go into the list */
}
static Node vec_pop(NodeVec *v) { v->len -= 1; return v->p[v->len]; }
static void node_drop(Node *n) {
    for (size_t i = 0; i < n->children.len; i++) node_drop(&n->children.p[i]);
    free(n->children.p);
}
static Node node_new(int64_t value) { Node n; memset(&n, 0, sizeof n); n.value = value; return n; }

/* Ember:
 *     root = Node(value=0)
 *     child = Node(value=1)
 *     child.children.push(Node(value=3))
 *     root.children.push(child)          # child now names root.children[child_at] (rule 3)
 *     root.children.pop()                # the popped value is a temporary, dropped here
 *     child.children.push(Node(value=7)) # Python: works, child is still a live object
 *     println(len(child.children), len(root.children))   # Python: 2 0 (py_attack_output.txt: store_pop_use 2 0)
 */
static void store_pop_use(int fixed) {
    Node root = node_new(0), child = node_new(1);
    vec_push(&child.children, node_new(3));
    size_t child_at = root.children.len;                         /* where push puts it (the report's code) */
    vec_push(&root.children, child);
    Node popped = vec_pop(&root.children);
    Node child_own;                                              /* the name's own storage (rule 2) */
    Node *childp;
    if (fixed) {                                                 /* fix: the name keeps the popped value */
        child_own = popped; childp = &child_own;
    } else {                                                     /* as written: pop ends nothing */
        node_drop(&popped);                                      /* the temporary is dropped */
        childp = &root.children.p[child_at];                     /* the report: "no count, no check" */
    }
    vec_push(&childp->children, node_new(7));
    printf("store_pop_use(%s): %zu %zu\n", fixed ? "fixed" : "as written", childp->children.len, root.children.len);
    if (fixed) node_drop(&child_own);
    node_drop(&root);
}

/* std/src/ffi.em, CppVector: `span(self)` views the C++ vector's elements; `push_back(mut self, v)`
 * writes no field of self, it calls the vector's push thunk, which may reallocate. Modelled with a
 * C vector of ints (what the thunk does to the elements is what std::vector does: move and free). */
typedef struct { int *p; size_t len, cap; } CVec;
static void cpp_push_back(CVec *v, int x) {                      /* the thunk */
    if (v->len == v->cap) {
        size_t cap = v->cap ? v->cap * 2 : 1;
        int *p = (int *)malloc(cap * sizeof(int));
        if (!p) abort();
        if (v->len) memcpy(p, v->p, v->len * sizeof(int));
        free(v->p);
        v->p = p; v->cap = cap;
    }
    v->p[v->len++] = x;
}
static void span_across_push(void) {
    CVec v = {0};
    cpp_push_back(&v, 5);                                        /* one element, capacity 1 */
    const int *s = v.p; size_t s_len = v.len;                    /* s = v.span() */
    cpp_push_back(&v, 7);                                        /* v.push_back(7): reallocates */
    printf("span_across_push: %d (len %zu)\n", s[0], s_len);     /* println(s[0]) */
    free(v.p);
}

/* Ember (accepted by the field-effects rules: Item.drop reads World.items and writes Item.hits; the
 * statement `w.items = []` reshapes World.items, and no loan is live across it):
 *     class Item:
 *         world: World
 *         hits: int = 0
 *         fn drop(self):
 *             for other in self.world.items:
 *                 other.hits += 1
 *     class World:
 *         items: Array[Item] = []
 *     fn reset(w: World):
 *         w.items = []            # Python (py_attack_output.txt): the drops walk the new, empty list
 */
typedef struct World World;
typedef struct Item { uint32_t strong; int64_t hits; World *world; } Item;
typedef struct { Item **p; size_t len, cap; } ItemVec;
struct World { ItemVec items; };
static int walked;
static void item_release(Item *it);
static void item_drop(Item *it) {                                /* fn drop(self) */
    ItemVec *l = &it->world->items;                              /* self.world.items, read now */
    for (size_t k = 0; k < l->len; k++) { l->p[k]->hits += 1; walked++; }
}
static void item_release(Item *it) {
    if (it->strong > 1) { it->strong -= 1; return; }
    it->strong = 0;
    item_drop(it);
    free(it);
}
static void drop_order(int detach_first) {
    World *w = (World *)calloc(1, sizeof(World));
    w->items.p = (Item **)malloc(3 * sizeof(Item *)); w->items.cap = 3;
    for (int k = 0; k < 3; k++) {
        Item *it = (Item *)calloc(1, sizeof(Item)); it->strong = 1; it->world = w;
        w->items.p[w->items.len++] = it;
    }
    ItemVec empty = {0};
    if (detach_first) {                                          /* the fix: old out, new in, then drop */
        ItemVec old = w->items; w->items = empty;
        for (size_t k = 0; k < old.len; k++) item_release(old.p[k]);
        free(old.p);
    } else {                                                     /* release the old value, then store */
        for (size_t k = 0; k < w->items.len; k++) item_release(w->items.p[k]);
        free(w->items.p);
        w->items = empty;
    }
    printf("drop_order(%s): the drops walked %d elements\n", detach_first ? "detach first" : "drop then store", walked);
    free(w);
}

/* Ember (A18):
 *     struct Game:
 *         width: int
 *         assets: Array[String]
 *         fn init(mut self, width: int):
 *             self.width = width
 *             self.load()                    # sets assets
 *         fn load(mut self):
 *             self.assets = [f"atlas@{self.width}px"]
 */
typedef struct { char **p; size_t len, cap; } StrVec;
typedef struct { int64_t width; StrVec assets; } Game;
static void strvec_drop(StrVec *v) { for (size_t k = 0; k < v->len; k++) free(v->p[k]); free(v->p); }
static StrVec atlas_list(int64_t width) {
    StrVec v; v.p = (char **)malloc(sizeof(char *)); v.len = 1; v.cap = 1;
    v.p[0] = (char *)malloc(32); snprintf(v.p[0], 32, "atlas@%lldpx", (long long)width);
    return v;
}
__declspec(noinline) static void game_load(Game *self) {          /* load, as a method of a finished Game */
    StrVec nv = atlas_list(self->width);
    strvec_drop(&self->assets);                                  /* assignment drops the old value */
    self->assets = nv;
}
__declspec(noinline) static void game_load_init(Game *self) {     /* the fix: load's copy for init */
    self->assets = atlas_list(self->width);                      /* a first store: nothing to drop */
}
static void init_helper(int fixed) {
    Game g;
    memset(&g, 0xBE, sizeof g);                                  /* what an unset stack slot may hold */
    g.width = 640;                                               /* self.width = width */
    if (fixed) game_load_init(&g); else game_load(&g);           /* self.load() */
    printf("init_helper(%s): %s\n", fixed ? "init copy" : "as written", g.assets.p[0]);
    strvec_drop(&g.assets);
}

int main(int argc, char **argv) {
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    int mode = argc > 1 ? atoi(argv[1]) : 0;
    if (mode == 1) store_pop_use(0);
    else if (mode == 2) store_pop_use(1);
    else if (mode == 3) span_across_push();
    else if (mode == 4) drop_order(0);
    else if (mode == 5) drop_order(1);
    else if (mode == 6) init_helper(0);
    else if (mode == 7) init_helper(1);
    else printf("usage: attacks.exe 1..7\n");
    return 0;
}
