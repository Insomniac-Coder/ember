/* Self-check (not timed) of the view loop's cursor rule used by ember_direct.c and ember_registry.c:
   a loop walks a pool backwards; its body destroys the current entity, destroys other entities
   (some already visited, some not yet reached), and creates entities. Rule under test: every entity
   that exists when the loop starts and is not destroyed before the loop reaches it is visited exactly
   once; an entity destroyed before the loop reaches it is never visited; created entities are not
   visited. Exit code 0 and "cursor-check ok" when the rule holds for every seed. */
#include "../harness.h"
#define assert(c) do { if (!(c)) { printf("cursor-check FAILED at line %d\n", __LINE__); exit(1); } } while (0)
#define NONE 0xFFFFFFFFu
#define MAXID 200000
typedef struct { uint32_t *sparse, *ents; size_t n, cap, cursor; } Pool;
static uint32_t visits[MAXID], gone_unvisited[MAXID], at_start[MAXID];

static void add(Pool *p, uint32_t id) {
    if (p->n == p->cap) { p->cap = p->cap ? p->cap * 2 : 4; p->ents = realloc(p->ents, p->cap * 4); }
    p->sparse[id] = (uint32_t)p->n; p->ents[p->n++] = id;
}
static void del(Pool *p, uint32_t id) {                  /* the rule from ember_direct.c */
    uint32_t i = p->sparse[id];
    assert(i != NONE && i < p->n);
    if (i < p->cursor) {
        size_t c = --p->cursor;
        uint32_t mc = p->ents[c];
        p->ents[i] = mc; p->sparse[mc] = i; i = (uint32_t)c;
    }
    size_t last = --p->n;
    uint32_t moved = p->ents[last];
    p->ents[i] = moved; p->sparse[moved] = i; p->sparse[id] = NONE;
}

int main(void) {
    h_start();
    for (uint64_t seed = 1; seed <= 200; seed++) {
        h_seed(seed);
        Pool p = {0};
        p.sparse = malloc(MAXID * 4);
        for (uint32_t i = 0; i < MAXID; i++) { p.sparse[i] = NONE; visits[i] = gone_unvisited[i] = at_start[i] = 0; }
        uint32_t next = 0, n0 = 100 + h_rand() % 400;
        for (uint32_t i = 0; i < n0; i++) { at_start[next] = 1; add(&p, next++); }
        for (size_t j = p.n; j-- > 0;) {
            uint32_t id = p.ents[j];
            visits[id]++;
            uint32_t r = h_rand() % 10;
            p.cursor = j;                                 /* published before calls that change p */
            if (r < 3 && p.n > 1) {                       /* destroy some other entity */
                uint32_t k = h_rand() % (uint32_t)p.n, other = p.ents[k];
                if (other != id) { if (k < j) gone_unvisited[other] = 1; del(&p, other); }
            }
            if (r == 3) del(&p, id);                      /* destroy the current one */
            if (r >= 4 && r < 6) add(&p, next++);         /* create */
            j = p.cursor;                                 /* read back */
            assert(j <= p.n);
        }
        for (uint32_t i = 0; i < next; i++) {
            if (!at_start[i]) assert(visits[i] == 0);             /* created during the loop */
            else if (gone_unvisited[i]) assert(visits[i] == 0);   /* destroyed before its turn */
            else assert(visits[i] == 1);                          /* everyone else: exactly once */
        }
        free(p.sparse); free(p.ents);
    }
    printf("cursor-check ok\n");
    return 0;
}
