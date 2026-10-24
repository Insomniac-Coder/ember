/* Attack on the ECS answer's cursor rule (phase3b-containers.md section 6.3 rule 4).
   cursor_check.c (the answer's own self-check) lets one iteration either destroy the current entity
   OR destroy one other entity, never both. A game destroys both: an explosion removes the current
   entity and its neighbours. This check lets one iteration destroy the current entity and up to three
   others (some visited, some not yet reached), in random order, and create entities.
   It checks the pool itself after every destroy (every ents[k] maps back to k through sparse) and the
   rule "every entity alive at the start and not destroyed before its turn is visited exactly once".
   Built twice:  -DDEL_ANSWER  the del() of ember_direct.c / cursor_check.c, as written
                 (default)     the same del() with the fix: skip the second move when the last
                               not-yet-visited slot is also the last slot (ember_registry.c's guard). */
#include "../harness.h"
#define FAIL(...) do { printf(__VA_ARGS__); printf("\n"); exit(1); } while (0)
#define NONE 0xFFFFFFFFu
#define MAXID 400000
typedef struct { uint32_t *sparse, *ents; size_t n, cap, cursor; } Pool;
static uint32_t visits[MAXID], gone_unvisited[MAXID], at_start[MAXID], alive[MAXID];

static void add(Pool *p, uint32_t id) {
    if (p->n == p->cap) { p->cap = p->cap ? p->cap * 2 : 4; p->ents = realloc(p->ents, p->cap * 4); }
    p->sparse[id] = (uint32_t)p->n; p->ents[p->n++] = id;
}
static void del(Pool *p, uint32_t id) {
    uint32_t i = p->sparse[id];
    if (!(i != NONE && i < p->n)) FAIL("del: entity %u has no valid slot (sparse=%u, n=%zu): a correct program panics", id, i, p->n);
    if (i < p->cursor) {
        size_t c = --p->cursor;
        uint32_t mc = p->ents[c];
        p->ents[i] = mc; p->sparse[mc] = i; i = (uint32_t)c;
    }
    size_t last = --p->n;
#ifdef DEL_ANSWER
    uint32_t moved = p->ents[last];                       /* as written in ember_direct.c */
    p->ents[i] = moved; p->sparse[moved] = i; p->sparse[id] = NONE;
#else
    if (i != last) { uint32_t moved = p->ents[last]; p->ents[i] = moved; p->sparse[moved] = i; }
    p->sparse[id] = NONE;
#endif
}
static void check_pool(const Pool *p, uint64_t seed) {
#ifdef NO_POOL_CHECK
    (void)p; (void)seed; return;
#endif
    for (size_t k = 0; k < p->n; k++)
        if (p->sparse[p->ents[k]] != k)
            FAIL("seed %llu: pool broken: ents[%zu]=%u but sparse says %u", (unsigned long long)seed, k, p->ents[k], p->sparse[p->ents[k]]);
}

int main(void) {
    h_start();
    for (uint64_t seed = 1; seed <= 2000; seed++) {
        h_seed(seed);
        Pool p = {0};
        p.sparse = malloc(MAXID * 4);
        for (uint32_t i = 0; i < MAXID; i++) { p.sparse[i] = NONE; visits[i] = gone_unvisited[i] = at_start[i] = alive[i] = 0; }
        uint32_t next = 0, n0 = 5 + h_rand() % 400;
        for (uint32_t i = 0; i < n0; i++) { at_start[next] = alive[next] = 1; add(&p, next++); }
        for (size_t j = p.n; j-- > 0;) {
            uint32_t id = p.ents[j];
            visits[id]++;
            p.cursor = j;                                 /* published before calls that change p */
            uint32_t r = h_rand() % 10;
            if (r < 4) {                                  /* an explosion: up to 3 others and maybe the current one, any order */
                int kill_current = h_rand() % 2, others = 1 + h_rand() % 3, cur_at = h_rand() % (others + 1);
                for (int step = 0; step <= others; step++) {
                    if (step == cur_at) { if (kill_current && alive[id]) { alive[id] = 0; del(&p, id); check_pool(&p, seed); } continue; }
                    if (p.n == 0) break;
                    uint32_t k = h_rand() % (uint32_t)p.n, other = p.ents[k];
                    if (other == id) continue;
                    if (k < p.cursor) gone_unvisited[other] = 1;
                    alive[other] = 0; del(&p, other); check_pool(&p, seed);
                }
            }
            if (r >= 4 && r < 6) { alive[next] = 1; add(&p, next++); }   /* create */
            j = p.cursor;                                 /* read back */
            if (j > p.n) FAIL("seed %llu: cursor past the end", (unsigned long long)seed);
        }
        for (uint32_t i = 0; i < next; i++) {
            if (!at_start[i]) { if (visits[i]) FAIL("seed %llu: created entity %u visited", (unsigned long long)seed, i); }
            else if (gone_unvisited[i]) { if (visits[i]) FAIL("seed %llu: entity %u destroyed before its turn but visited", (unsigned long long)seed, i); }
            else if (visits[i] != 1) FAIL("seed %llu: entity %u visited %u times", (unsigned long long)seed, i, visits[i]);
        }
        free(p.sparse); free(p.ents);
    }
#ifdef DEL_ANSWER
    printf("cursor-check2 (del as written) ok\n");
#else
    printf("cursor-check2 (del fixed) ok\n");
#endif
    return 0;
}
