/* lead_weakview.c: the lead's two variants of weakview (same program, same work, same checksum as
   weakview.h) that KEEP the view's count (+1 at the opening, -1 when the view ends) on every opening
   of a `thread_shared` Player through a `weakref` link, and make it cheaper.

   -DVARIANT_FADD   one locked add each way instead of a load + compare-exchange loop. The count word
                    has a DEAD bit (bit 31). An opening adds 1 with one `lock xadd`; if the word it
                    saw has DEAD, it takes the 1 back and the link counts as empty. A -1 that leaves
                    the count at 0 tries to swap 0 for DEAD; only then has the object ended (its drop,
                    then the owners' unit of the weak count). If an opening's +1 came in first, the
                    swap fails and that opening's own -1 tries again. An opening never sees an ended
                    object, and an object never ends while an opening holds its +1.
   -DVARIANT_BIASED biased counting (Choi, Shull and Torrellas, "Biased Reference Counting", PACT
                    2018). Each object records the thread that made it. That thread's +1 and -1 go to
                    a count only it ever writes (`biased`), with plain adds; every other thread's go to
                    a second, locked count (`shared`, with a MERGED and a DEAD bit). The object is alive
                    while biased + shared > 0. When `biased` reaches 0 its thread merges the two (sets
                    MERGED with one compare-exchange; if `shared` is 0 then, the object has ended).
                    A -1 from another thread that would take `shared` below 0 is handed to the making
                    thread's queue (the paper's protocol); this workload has one thread, so it times the
                    making thread's path, which is the common one. The other threads' paths are written
                    out below but not exercised here: they need their own test before any proposal.

   -DVARIANT_BIASED2 the same counting, emitted better: (1) the thread's id is read once per function
                    (it cannot change while the function runs); (2) the making thread writes `owner = 0`
                    when it merges, so "made here and not merged" is one compare of `owner` with the id
                    (only the making thread ever changes `owner`, and only to 0, so another thread's read
                    compares unequal before and after); (3) a view ends on the count it was opened on,
                    which the opening returns, so the -1 tests nothing but the count reaching 0.

   Header (OBJ-1 would change for BIASED and BIASED2): {biased u32, type index u32, weak u32, shared u32, owner thread
   u32, pad} = 24 bytes, so a Player is 40 bytes instead of 32. */
#include "m1rt.h"

#if defined(VARIANT_FADD)
#define NAME "lead-weakview-fadd"
#elif defined(VARIANT_BIASED)
#define NAME "lead-weakview-biased"
#elif defined(VARIANT_BIASED2)
#define NAME "lead-weakview-biased2"
#else
#error define VARIANT_FADD, VARIANT_BIASED or VARIANT_BIASED2
#endif

#define WP (1u << 14)
#define WC (1u << 20)
#define WF 64u
#define WMULT 40503u
#define PLAYER_TYPE_INDEX 7u
#define DEAD 0x80000000u

#if defined(VARIANT_FADD)
typedef struct { uint32_t count, type_index, weak, pad; } ObjHeader;           /* 16 bytes, as the model */
#else
#define MERGED 0x40000000u
#define SHARED_MASK 0x3FFFFFFFu
typedef struct { uint32_t biased, type_index, weak, shared, owner, pad; } ObjHeader;   /* 24 bytes */
#endif
typedef struct Player { ObjHeader h; int64_t x, id; } Player;
typedef struct { Player *target; } Link;
static Slab pslab = {0, 0, 0, sizeof(Player)};
static Player **players;
static volatile size_t vlen = WP;
static size_t players_len;

static NOINLINE void block_free(Player *p) { slab_free(&pslab, p); }
static inline void weak_add(Player *p) {
    if (UNLIKELY(AT_ADD32(&p->h.weak, 1) >= 0x7FFFFFFFu))
        e_stop("too many `weakref` links to a `Player` (2,147,483,647)");
}
static inline void weak_sub(Player *p) {
    if (AT_ADD32(&p->h.weak, -1) == 1) block_free(p);
}
static NOINLINE void object_ended(Player *p) {   /* its drop: none; then the owners' unit of the weak count */
    weak_sub(p);
}

#if defined(VARIANT_FADD)
static Player *new_player(uint32_t j, int64_t id) {
    Player *p = (Player *)slab_alloc(&pslab);
    p->h.count = 1; p->h.type_index = PLAYER_TYPE_INDEX; p->h.weak = 1; p->h.pad = 0;
    p->x = j & 1023; p->id = id;
    return p;
}
static NOINLINE void try_end(Player *p) {
    if (AT_CAS32(&p->h.count, 0, DEAD) == 0) object_ended(p);   /* else an opening came in first: its -1 retries */
}
static inline void owner_sub(Player *p) {
    if (AT_ADD32(&p->h.count, -1) == 1) try_end(p);
}
static inline const Player *link_open(const Link *l) {
    Player *p = l->target;
    if (p == NULL) return NULL;
    uint32_t v = AT_ADD32(&p->h.count, 1);                     /* the view's +1: one locked add */
    if (UNLIKELY(v & DEAD)) { AT_ADD32(&p->h.count, -1); return NULL; }   /* ended: give it back */
    if (UNLIKELY(v >= 0x7FFFFFFEu)) e_stop("too many owners of a `Player` (2,147,483,647)");
    return p;
}
static inline void view_end(const Player *p) { owner_sub((Player *)p); }   /* the view's -1 */

#elif defined(VARIANT_BIASED)
static inline uint32_t my_thread(void) { return __readgsdword(0x48); }   /* the thread's id, from its TEB */
static Player *new_player(uint32_t j, int64_t id) {
    Player *p = (Player *)slab_alloc(&pslab);
    p->h.biased = 1; p->h.type_index = PLAYER_TYPE_INDEX; p->h.weak = 1; p->h.shared = 0;
    p->h.owner = my_thread(); p->h.pad = 0;
    p->x = j & 1023; p->id = id;
    return p;
}
/* The making thread: its count reached 0. Merge: set MERGED; if `shared` holds no unit, the object ended. */
static NOINLINE void biased_zero(Player *p) {
    uint32_t s = AT_LOAD32(&p->h.shared);
    for (;;) {
        uint32_t seen = AT_CAS32(&p->h.shared, s, s | MERGED | ((s & SHARED_MASK) == 0 ? DEAD : 0));
        if (seen == s) break;
        s = seen;
    }
    if ((s & SHARED_MASK) == 0) object_ended(p);
}
/* Another thread's paths (not run by this one-thread workload). */
static NOINLINE const Player *shared_open(Player *p) {
    uint32_t v = AT_ADD32(&p->h.shared, 1);
    if (v & DEAD) { AT_ADD32(&p->h.shared, -1); return NULL; }
    if (UNLIKELY((v & SHARED_MASK) >= SHARED_MASK - 1)) e_stop("too many owners of a `Player` (2,147,483,647)");
    return p;
}
static NOINLINE void shared_sub(Player *p) {
    uint32_t v = AT_ADD32(&p->h.shared, -1);
    if ((v & MERGED) && (v & SHARED_MASK) == 1) {               /* the last unit after the merge */
        if (AT_CAS32(&p->h.shared, MERGED, MERGED | DEAD) == MERGED) object_ended(p);
    }
    /* A -1 that takes `shared` below 0 before the merge goes to the making thread's queue (the paper's
       protocol); that thread later adds it to `biased`. Not modelled: this workload never does it. */
}
static inline const Player *link_open(const Link *l) {
    Player *p = l->target;
    if (p == NULL) return NULL;
    if (p->h.owner == my_thread() && !(AT_LOAD32(&p->h.shared) & MERGED)) {   /* the making thread */
        if (p->h.biased == 0) return NULL;                     /* its owners have all ended */
        if (UNLIKELY(p->h.biased == 0xFFFFFFFFu)) e_stop("too many owners of a `Player` (4,294,967,295)");
        p->h.biased += 1;                                      /* the view's +1: a plain add */
        return p;
    }
    return shared_open(p);
}
static inline void view_end(const Player *p) {
    Player *q = (Player *)p;
    if (q->h.owner == my_thread() && !(AT_LOAD32(&q->h.shared) & MERGED)) {
        if (--q->h.biased == 0) biased_zero(q);                /* the view's -1: a plain subtract */
    } else {
        shared_sub(q);
    }
}
static inline void owner_sub(Player *p) { view_end(p); }       /* the slot's owner ends, on the making thread */

#else  /* VARIANT_BIASED2 */
static uint32_t tid;                                           /* read once at the top of main (see (1)) */
static inline uint32_t my_thread(void) { return __readgsdword(0x48); }
static Player *new_player(uint32_t j, int64_t id) {
    Player *p = (Player *)slab_alloc(&pslab);
    p->h.biased = 1; p->h.type_index = PLAYER_TYPE_INDEX; p->h.weak = 1; p->h.shared = 0;
    p->h.owner = tid; p->h.pad = 0;
    p->x = j & 1023; p->id = id;
    return p;
}
static NOINLINE void biased_zero(Player *p) {                  /* the making thread's count reached 0: merge */
    p->h.owner = 0;                                            /* (2): from now on every thread uses `shared` */
    uint32_t s = AT_LOAD32(&p->h.shared);
    for (;;) {
        uint32_t seen = AT_CAS32(&p->h.shared, s, s | MERGED | ((s & SHARED_MASK) == 0 ? DEAD : 0));
        if (seen == s) break;
        s = seen;
    }
    if ((s & SHARED_MASK) == 0) object_ended(p);
}
static NOINLINE int shared_open(Player *p) {                   /* another thread, or after the merge */
    uint32_t v = AT_ADD32(&p->h.shared, 1);
    if (v & DEAD) { AT_ADD32(&p->h.shared, -1); return 0; }
    if (UNLIKELY((v & SHARED_MASK) >= SHARED_MASK - 1)) e_stop("too many owners of a `Player` (2,147,483,647)");
    return 2;
}
static NOINLINE void shared_sub(Player *p) {
    uint32_t v = AT_ADD32(&p->h.shared, -1);
    if ((v & MERGED) && (v & SHARED_MASK) == 1) {
        if (AT_CAS32(&p->h.shared, MERGED, MERGED | DEAD) == MERGED) object_ended(p);
    }
}
/* Returns 0 when the link is empty or the object ended, 1 when the +1 went to `biased`, 2 to `shared`. */
static inline int link_open2(const Link *l, Player **out) {
    Player *p = l->target;
    *out = p;
    if (p == NULL) return 0;
    if (p->h.owner == tid) {                                   /* (2): made here, not merged: one compare */
        if (UNLIKELY(p->h.biased == 0xFFFFFFFFu)) e_stop("too many owners of a `Player` (4,294,967,295)");
        p->h.biased += 1;                                      /* the view's +1: a plain add */
        return 1;
    }
    return shared_open(p);
}
static inline void view_end2(Player *p, int how) {             /* (3): the -1 goes where the +1 went */
    if (how == 1) { if (--p->h.biased == 0) biased_zero(p); }
    else shared_sub(p);
}
static inline void owner_sub(Player *p) { view_end2(p, p->h.owner == tid ? 1 : 2); }
#endif

static void world_init(void) {
    players = (Player **)malloc(WP * sizeof(Player *));
    if (!players) abort();
    for (uint32_t j = 0; j < WP; j++) players[j] = new_player(j, j);
    players_len = vlen;
}
static inline void link_set(Link *l, uint32_t s) {
    if (UNLIKELY(s >= players_len)) e_stop("index out of bounds");
    Player *n = players[s];
    weak_add(n);
    l->target = n;
}
static inline void link_retarget(Link *l, uint32_t s) {
    Player *old = l->target;
    link_set(l, s);
    if (old) weak_sub(old);
}
static inline void replace_player(uint32_t j, int64_t id) {
    Player *n = new_player(j, id);
    Player *old = players[j];
    players[j] = n;
    owner_sub(old);
}

int main(void) {
    h_start();
#if defined(VARIANT_BIASED2)
    tid = my_thread();                                         /* (1) */
#endif
    world_init();
    Link *cams = (Link *)malloc(WC * sizeof(Link));
    if (!cams) abort();
    for (uint32_t i = 0; i < WC; i++) link_set(&cams[i], (i * WMULT) & (WP - 1));
    uint64_t acc = 0, fails = 0;
    for (uint32_t f = 0; f < WF; f++) {
        uint32_t r = f & 63;
        for (uint32_t j = r; j < WP; j += 64) replace_player(j, (int64_t)WP * (f + 1) + j);
        for (uint32_t i = 0; i < WC; i++) {
#if defined(VARIANT_BIASED2)
            Player *p;
            int how = link_open2(&cams[i], &p);
            if (how) {
                acc += (uint64_t)p->x;
                view_end2(p, how);
            } else {
#else
            const Player *p = link_open(&cams[i]);
            if (p) {
                acc += (uint64_t)p->x;
                view_end(p);
            } else {
#endif
                link_retarget(&cams[i], (i * WMULT) & (WP - 1));
                fails += 1;
            }
        }
    }
    h_report(NAME, acc + fails);
    return 0;
}
