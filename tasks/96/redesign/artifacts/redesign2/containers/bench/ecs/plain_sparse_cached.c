/* Plain C, EnTT layout, VARIANT of plain_sparse.c with the same hand optimisation the Ember model
   gets from the language: the pools' data pointers and lengths are held in locals on the common path
   and read again after the rare branch (the only code that changes the pools). No checks. This is the
   fairest "best plain C" for ember_direct.c. */
#include "../harness.h"
#define N 1000000
#define NONE 0xFFFFFFFFu
typedef struct { float x, y; } V2;
typedef struct { uint32_t *sparse, *ents; V2 *data; size_t n, cap, scap; } Pool;

static void grow_sparse(Pool *p, uint32_t id) {
    if (id < p->scap) return;
    size_t nc = p->scap ? p->scap : 1024;
    while (nc <= id) nc *= 2;
    p->sparse = realloc(p->sparse, nc * 4);
    for (size_t i = p->scap; i < nc; i++) p->sparse[i] = NONE;
    p->scap = nc;
}
static void add(Pool *p, uint32_t id, V2 v) {
    grow_sparse(p, id);
    if (p->n == p->cap) {
        p->cap = p->cap ? p->cap * 2 : 4;
        p->ents = realloc(p->ents, p->cap * 4); p->data = realloc(p->data, p->cap * sizeof(V2));
    }
    p->sparse[id] = (uint32_t)p->n; p->ents[p->n] = id; p->data[p->n] = v; p->n++;
}
static void del(Pool *p, uint32_t id) {
    uint32_t i = p->sparse[id], last = p->ents[--p->n];
    p->ents[i] = last; p->data[i] = p->data[p->n]; p->sparse[last] = i; p->sparse[id] = NONE;
}

int main(void) {
    h_start();
    Pool pos = {0}, vel = {0};
    for (uint32_t i = 0; i < N; i++) {
        add(&pos, i, (V2){(float)(i % 1000), (float)(i / 1000)});
        if (i % 2 == 0) add(&vel, i, (V2){(float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f});
    }
    uint32_t next = N;
    for (uint32_t s = 0; s < 100; s++) {
        uint32_t *v_ents = vel.ents; V2 *v_data = vel.data;
        uint32_t *p_sparse = pos.sparse; size_t p_scap = pos.scap; V2 *p_data = pos.data;
        for (size_t j = vel.n; j-- > 0;) {
            uint32_t id = v_ents[j], pi = id < p_scap ? p_sparse[id] : NONE;
            if (pi == NONE) continue;
            V2 *p = &p_data[pi], *v = &v_data[j];
            p->x += v->x; p->y += v->y;
            if (id % 100 == s) {
                del(&pos, id); del(&vel, id);
                uint32_t e = next++;
                add(&pos, e, (V2){0, 0}); add(&vel, e, (V2){1, 1});
                v_ents = vel.ents; v_data = vel.data;
                p_sparse = pos.sparse; p_scap = pos.scap; p_data = pos.data;
            }
        }
    }
    uint64_t sum = 0;
    for (size_t j = 0; j < pos.n; j++)
        sum += (uint64_t)((int64_t)(pos.data[j].x * 4.0f) + (int64_t)(pos.data[j].y * 4.0f));
    h_report("plain-w1-sparse-cached", sum + pos.n);
    return 0;
}
