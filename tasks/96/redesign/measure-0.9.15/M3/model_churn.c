/* W3 "churn", the model: what Ember emits for this program under spec 0.9.15_Hardened_1 as written.

   The Ember program (the pool is the program's own memory manager, a kind with several instances
   that frees single values):

       struct Obj:
           id: int
           x: int32
           y: int32
           hp: int32
           team: int32
           score: int

       struct Pool: ...                      # one block size, a free list, slabs of 64 KB
       unsafe extend Pool implements Allocator:
           fn allocate(self, layout: Layout) -> *mut u8: ...           # the free list's head, else the slab
           fn deallocate(self, p: *mut u8, layout: Layout): ...        # onto the free list
           fn reallocate(self, p: *mut u8, old: Layout, new_size: usize) -> *mut u8: ...

       fn make(ref mut pools: Array[Pool], i: int, r: u32) -> owns Obj using Pool:
           team = r & 1
           return owns Obj(id=next_id(), x=i, y=(r >> 8) & 1023, hp=1 + (r >> 20) % 16,
                           team=team, score=0) using pools[team]

       fn main():
           pools = [Pool(block=32), Pool(block=32)]
           slots: Array[owns Obj using Pool] = []
           for i in 0..S: slots.push(make(ref mut pools, i, rand()))
           for step in 0..STEPS:
               r = rand()
               i = (r >> 3) % S
               o = ref mut slots[i]
               o.hp -= (r >> 25) & 7
               o.score += o.x + o.y
               cs += o.score ^ o.id
               if o.hp <= 0:
                   cs += o.score
                   slots[i] = make(ref mut pools, i, rand())   # the old object ends: back to its pool

   What the rule makes it emit, against plain_churn_pool.c:
   - each `owns Obj using Pool` keeps its instance's address (R3, [ALC-1], verdict 47): a slot is
     {block, pool}, 16 bytes against C's 8;
   - each object's block comes from `pools[team].allocate(Layout(32, 8))` ([ALC-1]), a null result
     panics ([ALC-4]);
   - each object's end gives its block back through the kept word: `deallocate(word, p, Layout)`
     ([HEAP-1], R3: "so that its growth and its end go back to the instance it came from");
   - `slots[i]` and `pools[team]` are bounds-checked (today's checks, kept in every Ember form here).
   The pool is the program's own code, so its methods are compiled into the program (inline). */
#include "harness.h"
#include "m3_rt.h"

#define S (1u << 18)
#define STEPS (1u << 25)

typedef struct Obj { int64_t id; int32_t x, y, hp, team; int64_t score; } Obj;
typedef char size_check_Obj[sizeof(Obj) == 32 ? 1 : -1]; /* the layout M3.md states */
/* `owns Obj using Pool`: the block, and its instance's address (R3): 16 bytes */
typedef struct ObjBox { Obj *p; Pool *a; } ObjBox;
typedef char size_check_ObjBox[sizeof(ObjBox) == 16 ? 1 : -1];

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
    ObjBox b = { o, inst };                               /* the word kept (R3) */
    return b;
}

/* The end of an `owns Obj using Pool`: Obj has no drop; its block back to its instance through the
   kept word (R3, [ALC-2] deallocate). */
static inline void drop_obj(ObjBox b) { pool_free(b.a, b.p); }

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
    h_report("model_churn", cs);
    return 0;
}
