/* A4 workload: spawn and despawn inside the update loop, on a plain list of structs.
   1,000,000 enemies, 100 frames. Each frame visits every enemy in order (Python's list: the live
   length, so an enemy appended during the frame is visited in that frame): x += vx, hp -= 1; an
   enemy whose hp reaches 0 is removed and replaced by one new enemy appended at the end (hp 2..101,
   so a new enemy never dies in the frame it was born). Order is kept, as Python's list.remove keeps it.
   Checksum: sum of (int64)(x*4) + hp over the survivors, plus the length and the next id. */
#include "../harness.h"
#include <string.h>
#define N 1000000
#define FRAMES 100
typedef struct { uint32_t id; int32_t hp; float x, vx; } Enemy;   /* 16 bytes, plain data */
typedef struct { Enemy *data; size_t len, cap; } List;            /* Ember's Array layout: ptr, len, cap */

static void list_grow(List *l) {                                   /* the slow path of push */
    l->cap = l->cap ? l->cap * 2 : 4;
    l->data = realloc(l->data, l->cap * sizeof(Enemy));
}
static inline void list_push(List *l, Enemy e) {
    if (l->len == l->cap) list_grow(l);
    l->data[l->len++] = e;
}
static void fill(List *l) {
    h_seed(42);
    l->data = NULL; l->len = l->cap = 0;
    for (uint32_t i = 0; i < N; i++)
        list_push(l, (Enemy){i, 1 + (int32_t)(h_rand() % 100), (float)(i % 1000), (float)(i % 7) * 0.5f});
}
static uint64_t checksum(const List *l, uint32_t next) {
    uint64_t s = 0;
    for (size_t i = 0; i < l->len; i++) s += (uint64_t)((int64_t)(l->data[i].x * 4.0f) + l->data[i].hp);
    return s + l->len + next;
}
