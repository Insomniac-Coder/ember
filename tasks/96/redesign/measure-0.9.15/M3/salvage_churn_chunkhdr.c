/* W3 "churn", salvage (b) of verdict 47 for a memory manager that frees single values.

   The pool already starts every 64 KB slab with a header (its pool and the next slab, the list it
   frees at the end); its slabs come from VirtualAlloc, so they lie on 64 KB boundaries. A block's
   pool is then `((PoolSlab *)(p & ~0xFFFF))->pool`: one AND and one load, at the block's end only.
   The value keeps no word: a slot is the block's address alone, 8 bytes, as in plain C.

   Way (a) does not apply here: this kind does free single values, so an end must find its pool.
   This program is the model (model_churn.c) with only that change: the same allocate, the same
   free list, the same checks ([ALC-4], the bounds checks), the same order of making and ending.

   What it needs from the language: as salvage_trees_chunkhdr.c: one more unsafe promise the
   `Allocator` implementer makes ("each block's address finds its instance"), here kept by the
   slab's header and alignment; nothing in the program's text changes. A kind that does not make
   the promise keeps the word.

   Safety kept, exactly: the end goes back to the instance the block came from, by the kind's
   promise; E3063 (no value outlives its pool) and [ALC-5] read the making line, not the word.
   Cost: one load per end (the slab header: 64 slabs per 4 MB, usually in cache); memory: the header
   is already there, so none. */
#include "harness.h"
#include "m3_rt.h"

#define S (1u << 18)
#define STEPS (1u << 25)

typedef struct Obj { int64_t id; int32_t x, y, hp, team; int64_t score; } Obj;
typedef char size_check_Obj[sizeof(Obj) == 32 ? 1 : -1]; /* the layout M3.md states */
/* `owns Obj using Pool`: the block alone, 8 bytes; its pool is found from its address */
typedef struct ObjBox { Obj *p; } ObjBox;
typedef char size_check_ObjBox[sizeof(ObjBox) == 8 ? 1 : -1];

static Pool pools[2]; /* pools: Array[Pool], length 2 */
static int64_t next_id = 0;

static ObjBox make(uint32_t i, uint32_t r) {
    uint32_t team = r & 1;
    if (team >= 2) m3_panic("index out of bounds");      /* pools[team]: bounds check (today's) */
    Pool *inst = &pools[team];
    Obj *o = (Obj *)pool_alloc(inst);                     /* `using pools[team]`: allocate ([ALC-1]) */
    if (o == NULL) m3_panic("out of memory");             /* [ALC-4] */
    o->id = next_id++; o->x = (int32_t)i; o->y = (int32_t)((r >> 8) & 1023);
    o->hp = (int32_t)(1 + (r >> 20) % 16); o->team = (int32_t)team; o->score = 0;
    ObjBox b = { o };                                     /* no word */
    return b;
}

/* The end of an `owns Obj using Pool`: Obj has no drop; its block back to the instance its address
   finds ([ALC-2] deallocate, R3's purpose kept). */
static inline void drop_obj(ObjBox b) { pool_free(pool_of(b.p), b.p); }

int main(void) {
    h_start();
    h_seed(42);
    pool_init(&pools[0], sizeof(Obj));
    pool_init(&pools[1], sizeof(Obj));
    ObjBox *slots = (ObjBox *)malloc(S * sizeof(ObjBox)); /* Array[owns Obj using Pool], from Global */
    if (slots == NULL) m3_panic("out of memory");
    for (uint32_t i = 0; i < S; i++) slots[i] = make(i, h_rand());
    uint64_t cs = 0;
    for (uint32_t step = 0; step < STEPS; step++) {
        uint32_t r = h_rand();
        uint32_t i = (r >> 3) % S;
        if (i >= S) m3_panic("index out of bounds");      /* slots[i]: bounds check (today's) */
        Obj *o = slots[i].p;
        o->hp -= (int32_t)((r >> 25) & 7);
        o->score += o->x + o->y;
        cs += (uint64_t)(o->score ^ o->id);
        if (o->hp <= 0) {
            cs += (uint64_t)o->score;
            ObjBox fresh = make(i, h_rand());
            drop_obj(slots[i]);                           /* `slots[i] = ...` ends the old value */
            slots[i] = fresh;
        }
    }
    for (uint32_t i = 0; i < S; i++) drop_obj(slots[i]);
    free(slots);
    pool_destroy(&pools[0]);
    pool_destroy(&pools[1]);
    cs += (uint64_t)next_id;
    h_report("salvage_churn_chunkhdr", cs);
    return 0;
}
