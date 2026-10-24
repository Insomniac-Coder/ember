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
 *   attacks.exe 8   (second attack) names follow values, rule 3, "put in a map": the name keeps the address
 *                   of the map entry it went into; a later insertion grows the entry list (std's Map keeps
 *                   its entries in one list, std/src/collections.em push_new), which moves and frees it.
 *   attacks.exe 9   the fix: the name is the entry's position in that list (the containers group's rule
 *                   for list elements), so the entry is found again after the growth.
 *   attacks.exe 10  (second attack) a generator's frame holds a loan to the list it walks; between two
 *                   next() calls the caller stores the list into an object's field (names follow values:
 *                   `items` now names w.items) and appends there. The frame still points at the old place.
 *   attacks.exe 11  the save: `yield` counts as a call the compiler cannot see, so the generator walks by
 *                   position and re-reads the list at each resume; growth by the caller is allowed (Python's
 *                   meaning), moving the walked list while the generator lives stays refused (today's rule).
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

/* Ember (rule 3 with a map):
 *     d = Map[String, Cell]()
 *     c1 = Cell(x=0, tag="first")
 *     d["a"] = c1                      # c1 now names d["a"]
 *     for k in range(100): d[str(k)] = Cell(x=0, tag="more")   # insertions: the entry list grows
 *     c1.x = 5                          # Python (py_attack_output.txt, dict_grow_then_name): 5
 * Modelled with std's Map layout reduced to its entry list (entries are appended; the slot table that
 * finds them by hash is left out because only the entry list holds the values). */
typedef struct { int64_t x; char *tag; } Cell;
typedef struct { int64_t key; Cell value; } Entry;
typedef struct { Entry *p; size_t len, cap; } EntryVec;
static void entries_push(EntryVec *v, Entry e) {
    if (v->len == v->cap) {
        size_t cap = v->cap ? v->cap * 2 : 4;
        Entry *p = (Entry *)malloc(cap * sizeof(Entry));
        if (!p) abort();
        if (v->len) memcpy(p, v->p, v->len * sizeof(Entry));
        free(v->p);                                              /* the old entry block is freed */
        v->p = p; v->cap = cap;
    }
    v->p[v->len++] = e;
}
static Cell cell_new(const char *tag) { Cell c; c.x = 0; c.tag = _strdup(tag); return c; }
static void map_name_after_growth(int fixed) {
    EntryVec d = {0};
    entries_push(&d, (Entry){ -1, cell_new("first") });         /* d["a"] = c1 */
    size_t c1_at = 0;                                            /* the position the value went to */
    Cell *c1_addr = &d.p[0].value;                               /* the address the value went to */
    for (int64_t k = 0; k < 100; k++) entries_push(&d, (Entry){ k, cell_new("more") });
    if (fixed) d.p[c1_at].value.x = 5;                           /* the fix: found again by position */
    else c1_addr->x = 5;                                         /* as written: the kept address */
    printf("map_name_after_growth(%s): d[\"a\"].x = %lld\n", fixed ? "position" : "address", (long long)d.p[0].value.x);
    for (size_t i = 0; i < d.len; i++) free(d.p[i].value.tag);
    free(d.p);
}

/* Ember (gen_store_between.em / gen_push_between.em):
 *     gen fn walk(xs: Array[int]) -> Generator[int]:
 *         for x in xs: yield x
 *     items = [1, 2]; g = walk(items); println(g.next())
 *     w.items = items; w.items.push(99)            # mode 10 (accepted only if a store may move a walked list)
 *     items.push(99)                               # mode 11 (the save)
 *     println(g.next()); println(g.next())         # Python: 2 then 99 */
typedef struct { int64_t *p; size_t len, cap; } I64Vec;
static void i64_push(I64Vec *v, int64_t x) {
    if (v->len == v->cap) {
        size_t cap = v->cap ? v->cap * 2 : 2;
        int64_t *p = (int64_t *)malloc(cap * sizeof(int64_t));
        if (!p) abort();
        if (v->len) memcpy(p, v->p, v->len * sizeof(int64_t));
        free(v->p);
        v->p = p; v->cap = cap;
    }
    v->p[v->len++] = x;
}
typedef struct { I64Vec *xs; size_t at; } WalkFrame;              /* the generator's frame */
static int walk_next(WalkFrame *g, int64_t *out) {                 /* by position, re-reading the list */
    if (g->at >= g->xs->len) return 0;
    *out = g->xs->p[g->at++];
    return 1;
}
static void generator_across_store(int saved) {
    I64Vec items = {0}; i64_push(&items, 1); i64_push(&items, 2);
    I64Vec w_items = {0};
    WalkFrame g = { &items, 0 };
    int64_t v = 0;
    walk_next(&g, &v); printf("generator(%s): %lld", saved ? "saved" : "store accepted", (long long)v);
    if (saved) {
        i64_push(&items, 99);                                      /* items.push(99) */
    } else {
        w_items = items;                                           /* w.items = items: the header moves */
        i64_push(&w_items, 99);                                    /* w.items.push(99): reallocates */
    }
    while (walk_next(&g, &v)) printf(" %lld", (long long)v);
    printf("\n");
    free(saved ? items.p : w_items.p);
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
    else if (mode == 8) map_name_after_growth(0);
    else if (mode == 9) map_name_after_growth(1);
    else if (mode == 10) generator_across_store(0);
    else if (mode == 11) generator_across_store(1);
    else printf("usage: attacks.exe 1..7\n");
    return 0;
}
