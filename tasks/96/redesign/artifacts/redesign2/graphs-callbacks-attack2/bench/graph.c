/* graph.c: a directed graph whose edges are kept in step both ways (A8's graph answer). The Ember
   program (proposed syntax; the rules are in phase3b-graphs-callbacks-attack.md section 4.4):

       class City:
           id: int
           roads: Array[weak City] = []
           into: Array[weak City] = backlink(roads)    # who has a road to me; kept by the language

       cities = [City(id=i) for i in range(N)]          # the only owner of every city
       for c in cities:
           for _ in range(R): c.roads.push(cities[rand() % N])   # also pushes c into the target's `into`
       for p in range(PASSES):
           for c in cities:                             # the walk: a load per road
               for r in c.roads: total += r.id
           for _ in range(N // 100):                    # 1% of the cities are replaced
               k = rand() % N
               cities[k] = City(id=next_id)             # the old city dies: the language takes it out of
               for _ in range(R):                       # every `roads` and `into` list that names it
                   cities[k].roads.push(cities[rand() % N])

   The Python twin keeps `into` by hand and removes a dead city from its neighbours' lists by hand;
   so does the plain C here. Both builds share every line of the graph code:
     (no switch)  plain C: one malloc per city, lists {ptr, len, cap} growing from 4, no checks
     -DEMBER      what the rules emit: the same code, plus the lean header (count and type index, 8
                  bytes), bounds checks on `cities[...]` and on list removal, and the count of each city
                  held by `cities` (released when replaced; the death runs when it reaches zero).
   Overflow checks are left out, as in every model. */
#include "ember_model.h"
#define N 100000
#define R 8
#define PASSES 10

typedef struct City {
#ifdef EMBER
    uint32_t rc, type;
#endif
    int64_t id;
    EVec roads, into;
} City;

static City *city_new(int64_t id) {
    City *c = calloc(1, sizeof *c);
#ifdef EMBER
    c->rc = 1; c->type = 1;
#endif
    c->id = id;
    return c;
}
#ifdef EMBER
#define CHECK(i, n) e_bounds((i), (n))
#else
#define CHECK(i, n) ((void)0)
#endif
/* remove one occurrence of x from l (order kept), as Python's list.remove */
static inline void list_remove_value(EVec *l, City *x) {
    size_t k = 0;
    while (k < l->len && l->ptr[k] != x) k++;
    CHECK(k, l->len);
    memmove(&l->ptr[k], &l->ptr[k + 1], (l->len - k - 1) * sizeof(void *));
    l->len--;
}
/* a.roads.push(b): the language (or the C programmer) also pushes a into b.into */
static inline void road_push(City *a, City *b) { evec_push(&a->roads, b); evec_push(&b->into, a); }
/* a city dies: it leaves every list that names it, then its memory is freed */
static void city_die(City *x) {
    for (size_t i = 0; i < x->into.len; i++) {         /* every road y -> x */
        City *y = x->into.ptr[i];
        if (y != x) list_remove_value(&y->roads, x);
    }
    for (size_t i = 0; i < x->roads.len; i++) {        /* every road x -> z */
        City *z = x->roads.ptr[i];
        if (z != x) list_remove_value(&z->into, x);
    }
    free(x->roads.ptr); free(x->into.ptr); free(x);
}
static inline void city_release(City *x) {
#ifdef EMBER
    if (--x->rc == 0) city_die(x);
#else
    city_die(x);
#endif
}

int main(void) {
    h_start();
    h_seed(11);
    City **cities = malloc(N * sizeof *cities);
    for (int64_t i = 0; i < N; i++) cities[i] = city_new(i);
    for (int64_t i = 0; i < N; i++)
        for (int r = 0; r < R; r++) { size_t t = h_rand() % N; CHECK(t, N); road_push(cities[i], cities[t]); }
    int64_t next_id = N;
    uint64_t total = 0;
    for (int p = 0; p < PASSES; p++) {
        for (int64_t i = 0; i < N; i++) {
            City *c = cities[i];
            for (size_t k = 0; k < c->roads.len; k++) total += (uint64_t)((City *)c->roads.ptr[k])->id;
        }
        for (int j = 0; j < N / 100; j++) {
            size_t k = h_rand() % N; CHECK(k, N);
            City *old = cities[k];
            cities[k] = city_new(next_id++);
            city_release(old);
            for (int r = 0; r < R; r++) { size_t t = h_rand() % N; CHECK(t, N); road_push(cities[k], cities[t]); }
        }
    }
    for (int64_t i = 0; i < N; i++) total += (uint64_t)(cities[i]->roads.len * 3 + cities[i]->into.len);
#ifdef EMBER
    h_report("graph-ember", total);
#else
    h_report("graph-plain", total);
#endif
    return 0;
}
