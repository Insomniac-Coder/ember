/* D2 (generational references + stable stores), workload W1: ECS.
   What the D2 compiler would emit for tasks/96/redesign/phase3-design-D2.md section 3, W1:
   - std.ecs as EnTT sparse sets, but each pool's slots live in a reserved address range that grows
     in place (slots never move), with tombstones instead of swap-and-pop;
   - Entity = a 64-bit serial number that is never reused, so the owning entity stored in a slot is
     also that slot's generation: a component reference (slot address, entity) is stale exactly when
     slot.owner != entity;
   - inserting into a pool while a loop iterates it reuses a free slot only if the loop has already
     passed it (iter_limit), so a spawned entity is never visited by the running loop;
   - the view's component references are fresh each iteration and used before any structural change,
     so they carry no generation check (proof P5); sparse lookups keep their bounds check;
   - destroy() visits every pool through its interface table, as EnTT's registry does.
   Not modelled (outside the memory model, same for every design): integer overflow checks. */
#include "../harness.h"
#include <string.h>

typedef uint32_t u32;
typedef uint64_t u64;
typedef int64_t i64;

#define N 1000000u
#define STEPS 100u

static void d2_panic(const char *m) {
    fprintf(stderr, "panic: %s\n", m);
    fflush(stderr);
    ExitProcess(101);
}

/* A reserved address range that is committed as it grows: what it holds never moves. */
typedef struct { char *base; size_t reserved, committed; } Range;

static void range_init(Range *r, size_t reserve) {
    r->base = (char *)VirtualAlloc(NULL, reserve, MEM_RESERVE, PAGE_NOACCESS);
    if (!r->base) d2_panic("address space reservation failed");
    r->reserved = reserve;
    r->committed = 0;
}

static void range_ensure(Range *r, size_t bytes) {
    if (bytes <= r->committed) return;
    size_t chunk = (size_t)1 << 20;
    size_t want = (bytes + chunk - 1) & ~(chunk - 1);
    if (want > r->reserved) d2_panic("store capacity exhausted");
    if (!VirtualAlloc(r->base + r->committed, want - r->committed, MEM_COMMIT, PAGE_READWRITE))
        d2_panic("out of memory");
    r->committed = want;
}

/* ---- std.ecs ---- */

#define DEAD_OWNER UINT64_MAX
#define NO_SLOT UINT32_MAX

typedef struct { float x, y; } Position;
typedef struct { float dx, dy; } Velocity;

/* Every pool slot starts with its owner: the entity id, or DEAD_OWNER for a free slot. */
typedef struct { u64 owner; Position v; } PosSlot;
typedef struct { u64 owner; Velocity v; } VelSlot;

typedef struct Pool Pool;
typedef struct { void (*remove)(Pool *, u64); } AnyPoolVT; /* `interface AnyPool` */

struct Pool {
    const AnyPoolVT *vt;
    Range slots;     /* Slot[], never moves */
    Range sparse;    /* u32 per entity id: slot index + 1, 0 = absent */
    u64 sparse_len;  /* ids covered by `sparse` */
    u32 slot_size;
    u32 len;         /* slots in use (high-water mark) */
    u32 free_top;    /* intrusive stack of free slots */
    u32 iter_limit;  /* while a loop iterates this pool: reuse only free slots <= this */
    u32 count;       /* live components */
};

static void pool_init(Pool *p, const AnyPoolVT *vt, u32 slot_size) {
    p->vt = vt;
    range_init(&p->slots, (size_t)64 << 20);
    range_init(&p->sparse, (size_t)32 << 20);
    p->sparse_len = 0;
    p->slot_size = slot_size;
    p->len = 0;
    p->free_top = NO_SLOT;
    p->iter_limit = UINT32_MAX;
    p->count = 0;
}

static inline u64 *slot_owner(Pool *p, u32 s) { return (u64 *)(p->slots.base + (size_t)s * p->slot_size); }

/* sparse[e], bounds-checked: an id past the end is simply absent */
static inline u32 sparse_get(Pool *p, u64 e) {
    return e < p->sparse_len ? ((u32 *)p->sparse.base)[e] : 0;
}

static void sparse_cover(Pool *p, u64 e) {
    if (e < p->sparse_len) return;
    range_ensure(&p->sparse, (size_t)(e + 1) * sizeof(u32));
    p->sparse_len = e + 1;
}

/* Take a slot for a new component: a free one the running loop (if any) has passed, else append. */
static u32 pool_take_slot(Pool *p) {
    u32 s = p->free_top;
    if (s != NO_SLOT && s <= p->iter_limit) {
        p->free_top = *(u32 *)(p->slots.base + (size_t)s * p->slot_size + 8); /* next, kept in the value bytes */
        return s;
    }
    s = p->len++;
    range_ensure(&p->slots, (size_t)p->len * p->slot_size);
    return s;
}

static void pool_remove(Pool *p, u64 e) {
    u32 s1 = sparse_get(p, e);
    if (s1 == 0) return;
    ((u32 *)p->sparse.base)[e] = 0;
    u32 s = s1 - 1;
    char *sl = p->slots.base + (size_t)s * p->slot_size;
    *(u64 *)sl = DEAD_OWNER;            /* the slot's generation changes: references to it are stale */
    *(u32 *)(sl + 8) = p->free_top;     /* Position and Velocity need no drop */
    p->free_top = s;
    p->count--;
}

static const AnyPoolVT pos_vt = {pool_remove};
static const AnyPoolVT vel_vt = {pool_remove};

static PosSlot *pos_emplace(Pool *p, u64 e, Position v) {
    sparse_cover(p, e);
    u32 *sp = (u32 *)p->sparse.base + e;
    if (*sp != 0) d2_panic("emplace: the entity already has a Position");
    u32 s = pool_take_slot(p);
    PosSlot *sl = (PosSlot *)p->slots.base + s;
    sl->owner = e;
    sl->v = v;
    *sp = s + 1;
    p->count++;
    return sl;
}

static VelSlot *vel_emplace(Pool *p, u64 e, Velocity v) {
    sparse_cover(p, e);
    u32 *sp = (u32 *)p->sparse.base + e;
    if (*sp != 0) d2_panic("emplace: the entity already has a Velocity");
    u32 s = pool_take_slot(p);
    VelSlot *sl = (VelSlot *)p->slots.base + s;
    sl->owner = e;
    sl->v = v;
    *sp = s + 1;
    p->count++;
    return sl;
}

typedef struct {
    Range alive;      /* bit per entity id */
    u64 next_id;
    u64 alive_count;
    Pool *pools[2];   /* by compile-time component index: 0 Position, 1 Velocity */
    int npools;
} Registry;

static u64 reg_create(Registry *r) {
    u64 e = r->next_id++;
    range_ensure(&r->alive, (size_t)((e >> 6) + 1) * 8);
    ((u64 *)r->alive.base)[e >> 6] |= 1ull << (e & 63);
    r->alive_count++;
    return e;
}

static int reg_valid(Registry *r, u64 e) {
    return e < r->next_id && ((((u64 *)r->alive.base)[e >> 6] >> (e & 63)) & 1);
}

static void reg_destroy(Registry *r, u64 e) {
    if (!reg_valid(r, e)) d2_panic("destroy: the entity is not alive");
    for (int i = 0; i < r->npools; i++) r->pools[i]->vt->remove(r->pools[i], e);
    ((u64 *)r->alive.base)[e >> 6] &= ~(1ull << (e & 63));
    r->alive_count--;
}

int main(void) {
    h_start();

    Pool pos, vel;
    pool_init(&pos, &pos_vt, sizeof(PosSlot));
    pool_init(&vel, &vel_vt, sizeof(VelSlot));
    Registry reg;
    range_init(&reg.alive, (size_t)4 << 20);
    reg.next_id = 0;
    reg.alive_count = 0;
    reg.pools[0] = &pos;
    reg.pools[1] = &vel;
    reg.npools = 2;

    for (u32 i = 0; i < N; i++) {
        u64 e = reg_create(&reg);
        pos_emplace(&pos, e, (Position){(float)(i % 1000), (float)(i / 1000)});
        if (i % 2 == 0) vel_emplace(&vel, e, (Velocity){(float)(i % 7) * 0.5f, (float)(i % 5) * 0.25f});
    }

    for (u32 s = 0; s < STEPS; s++) {
        /* for e, p, v in reg.view[(Position, Velocity)](): iterate the smaller pool, probe the other */
        int lead_is_vel = vel.count <= pos.count;
        Pool *lead = lead_is_vel ? &vel : &pos;
        u32 n = lead->len; /* the loop visits the slots present when it began */
        for (u32 j = 0; j < n; j++) {
            u64 e = *slot_owner(lead, j);
            if (e == DEAD_OWNER) continue;
            PosSlot *p;
            VelSlot *v;
            if (lead_is_vel) {
                v = (VelSlot *)vel.slots.base + j;
                u32 ps = sparse_get(&pos, e);
                if (ps == 0) continue;
                p = (PosSlot *)pos.slots.base + (ps - 1);
            } else {
                p = (PosSlot *)pos.slots.base + j;
                u32 vs = sparse_get(&vel, e);
                if (vs == 0) continue;
                v = (VelSlot *)vel.slots.base + (vs - 1);
            }
            /* p and v are fresh this iteration and used before any structural change: no check */
            p->v.x += v->v.dx;
            p->v.y += v->v.dy;
            if (e % 100 == s) {
                lead->iter_limit = j; /* the cursor, published before calls that may insert into `lead` */
                reg_destroy(&reg, e);
                u64 ne = reg_create(&reg);
                pos_emplace(&pos, ne, (Position){0.0f, 0.0f});
                vel_emplace(&vel, ne, (Velocity){1.0f, 1.0f});
            }
        }
        lead->iter_limit = UINT32_MAX;
    }

    /* checksum: for e, p in reg.view[(Position,)]() */
    u64 sum = 0;
    for (u32 j = 0; j < pos.len; j++) {
        PosSlot *p = (PosSlot *)pos.slots.base + j;
        if (p->owner == DEAD_OWNER) continue;
        sum += (u64)(i64)(p->v.x * 4.0f) + (u64)(i64)(p->v.y * 4.0f);
    }
    sum += reg.alive_count;

    h_report("D2-w1", sum);
    return 0;
}
