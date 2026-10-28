/* W3 "churn", plain C with two pools: what a careful C programmer writes for small objects that die
   one at a time in two memory managers. No safety checks.
   The work every program of W3 does: 2^18 slots each own one 32-byte object, made in one of two pool
   instances (its team's); 2^25 steps each pick a random slot, update its object, and when the
   object's hp runs out end it (its block back to its own pool) and make a new one in a random team's
   pool. About 30% of steps end one object and make another. At the end every object ends.
   This is run 3's W3 pool (blocks on a free list) with two instances, the case R3's word exists for:
   a value's end must go back to the instance it came from.
   The C programmer needs no word: the object's own `team` field already says which pool it came
   from (`pools[o->team]`), knowledge a compiler does not have.
   Checksum: per step cs += score ^ id; per ended object cs += score; at the end cs += objects made. */
#include "harness.h"
#include "m3_rt.h"

#define S (1u << 18)
#define STEPS (1u << 25)

typedef struct Obj { int64_t id; int32_t x, y, hp, team; int64_t score; } Obj;
typedef char size_check_Obj[sizeof(Obj) == 32 ? 1 : -1]; /* the layout M3.md states */

static Pool pools[2];
static int64_t next_id = 0;

static Obj *make(uint32_t i, uint32_t r) {
    int32_t team = (int32_t)(r & 1);
    Obj *o = (Obj *)pool_alloc(&pools[team]);
    o->id = next_id++; o->x = (int32_t)i; o->y = (int32_t)((r >> 8) & 1023);
    o->hp = (int32_t)(1 + (r >> 20) % 16); o->team = team; o->score = 0;
    return o;
}

int main(void) {
    h_start();
    h_seed(42);
    pool_init(&pools[0], sizeof(Obj));
    pool_init(&pools[1], sizeof(Obj));
    Obj **slots = (Obj **)malloc(S * sizeof(Obj *));
    for (uint32_t i = 0; i < S; i++) slots[i] = make(i, h_rand());
    uint64_t cs = 0;
    for (uint32_t step = 0; step < STEPS; step++) {
        uint32_t r = h_rand();
        uint32_t i = (r >> 3) % S;
        Obj *o = slots[i];
        o->hp -= (int32_t)((r >> 25) & 7);
        o->score += o->x + o->y;
        cs += (uint64_t)(o->score ^ o->id);
        if (o->hp <= 0) {
            cs += (uint64_t)o->score;
            Obj *fresh = make(i, h_rand()); /* made before the old one ends, as Ember's and Rust's `=` */
            pool_free(&pools[o->team], o);
            slots[i] = fresh;
        }
    }
    for (uint32_t i = 0; i < S; i++) pool_free(&pools[slots[i]->team], slots[i]);
    free(slots);
    pool_destroy(&pools[0]);
    pool_destroy(&pools[1]);
    cs += (uint64_t)next_id;
    h_report("plain_churn_pool", cs);
    return 0;
}
