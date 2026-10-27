/* stress.c: does a way of counting `thread_shared` owners keep every promise under real threads?
   Build with -DCOUNT_ATOMIC (today's rule) or -DCOUNT_BIASED, and -DBRC_CHECK (blocks are poisoned, never
   freed, so any read of a freed block is seen).

   Usage: stress <threads per wave> <waves> <ops per thread> <seed>

   Each wave starts `threads` threads; the next wave starts when it has ended, so a later wave works on
   objects whose owner thread is gone. Every thread does random operations:
     make   a new object, held in one of the thread's own owner places (the old one there is dropped)
     copy   one more owner of an object it holds (own_inc)
     drop   one of its owners (own_dec)
     put    one of its owners into a global place, under that place's lock; the place's old owner is dropped
     take   the owner out of a global place, under the lock (a later drop happens on this thread)
     share  one more owner of the object in a global place, under the lock (own_inc while the place's owner
            keeps it alive)
     link   a `weakref` link, in one of the thread's own link places, to the object in a global place
            (under the lock) or to one it holds; the old link is ended
     open   `if link exists as v:` read the object while the view lives; it must be alive (canary LIVE)
     safe   a safe point (biased counting merges what other threads queued to this one)
   The global places model a `thread_shared` container behind a lock (Ember changes a thread_shared value
   only through a lock, an atomic or a channel: E7014).

   Checks: a view never sees an object that has ended (canary); an object ends exactly once (object_ended
   checks); a block is freed only after its object ended, and only once (block_free checks); at the end,
   after every place is emptied, every object made has ended and every block has been freed. */
#include "brc.h"

#define LOCAL_OWNERS 16
#define LOCAL_LINKS 16
#define GLOBAL_PLACES 256

typedef struct { volatile long lock; Obj *o; } Place;
static Place places[GLOBAL_PLACES];
static volatile long ended_objects, views_checked;
static long ops_per_thread;
static unsigned nplaces = GLOBAL_PLACES;   /* fewer places: more threads on the same objects */

static void lock(Place *p) { while (_InterlockedExchange(&p->lock, 1)) { while (p->lock) _mm_pause(); } }
static void unlock(Place *p) { _InterlockedExchange(&p->lock, 0); }

static void count_drop(Obj *o) { (void)o; _InterlockedIncrement(&ended_objects); }

typedef struct { uint64_t rng; BrcThread *t; } Worker;
static inline uint64_t rnd(Worker *w) {
    uint64_t x = w->rng; x ^= x << 13; x ^= x >> 7; x ^= x << 17; return w->rng = x;
}

static DWORD WINAPI worker_main(LPVOID arg) {
    Worker *w = (Worker *)arg;
    BrcThread *t = brc_thread_start();
    w->t = t;
#if !defined(COUNT_ATOMIC)
    brc_threads[t->id] = t;
#endif
    Obj *mine[LOCAL_OWNERS] = {0};
    Obj *links[LOCAL_LINKS] = {0};
    for (long n = 0; n < ops_per_thread; n++) {
        uint64_t r = rnd(w);
        unsigned op = (unsigned)(r % 100), a = (unsigned)((r >> 8) % LOCAL_OWNERS);
        unsigned b = (unsigned)((r >> 16) % LOCAL_OWNERS), g = (unsigned)((r >> 24) % nplaces);
        unsigned l = (unsigned)((r >> 32) % LOCAL_LINKS);
        if (op < 10) {                                   /* make */
            Obj *o = obj_new(t, r);
            if (mine[a]) own_dec(t, mine[a]);
            mine[a] = o;
        } else if (op < 18) {                            /* copy */
            if (mine[a] && a != b) { own_inc(t, mine[a]); if (mine[b]) own_dec(t, mine[b]); mine[b] = mine[a]; }
        } else if (op < 26) {                            /* drop */
            if (mine[a]) { own_dec(t, mine[a]); mine[a] = NULL; }
        } else if (op < 36) {                            /* put */
            if (mine[a]) {
                Place *p = &places[g];
                lock(p); Obj *old = p->o; p->o = mine[a]; unlock(p);
                mine[a] = NULL;
                if (old) own_dec(t, old);
            }
        } else if (op < 44) {                            /* take */
            Place *p = &places[g];
            lock(p); Obj *o = p->o; p->o = NULL; unlock(p);
            if (o) { if (mine[a]) own_dec(t, mine[a]); mine[a] = o; }
        } else if (op < 52) {                            /* share */
            Place *p = &places[g];
            lock(p); Obj *o = p->o; if (o) own_inc(t, o); unlock(p);
            if (o) { if (mine[a]) own_dec(t, mine[a]); mine[a] = o; }
        } else if (op < 62) {                            /* link */
            Obj *o = NULL;
            if (r & (1ull << 40)) {
                Place *p = &places[g];
                lock(p); o = p->o; if (o) weak_add(o); unlock(p);
            } else if (mine[a]) {
                o = mine[a]; weak_add(o);
            }
            if (o) { if (links[l]) weak_sub(links[l]); links[l] = o; }
        } else if (op < 97) {                            /* open */
            Obj *o = links[l];
            if (o) {
                int how = weak_open(t, o);
                if (how) {
                    if (o->canary != LIVE) brc_stop("a view saw an object that had ended");
                    volatile uint64_t v = o->value; (void)v;
                    if ((r >> 41) & 1) {                 /* while the view lives, drop or take something */
                        if (mine[b]) { own_dec(t, mine[b]); mine[b] = NULL; }
                    }
                    if (o->canary != LIVE) brc_stop("an object ended while a view held it");
                    _InterlockedIncrement(&views_checked);
                    view_end(t, o, how);
                }
            }
        } else {                                         /* safe */
            brc_safe_point(t);
        }
    }
    for (int i = 0; i < LOCAL_OWNERS; i++) if (mine[i]) own_dec(t, mine[i]);
    for (int i = 0; i < LOCAL_LINKS; i++) if (links[i]) weak_sub(links[i]);
    brc_thread_end(t);
    return 0;
}

int main(int argc, char **argv) {
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    if (argc != 5 && argc != 6) { fprintf(stderr, "usage: stress threads waves ops seed [places]\n"); return 2; }
    if (argc == 6) { nplaces = (unsigned)atoi(argv[5]); if (nplaces < 1 || nplaces > GLOBAL_PLACES) return 2; }
    int threads = atoi(argv[1]), waves = atoi(argv[2]);
    ops_per_thread = atol(argv[3]);
    uint64_t seed = (uint64_t)_strtoui64(argv[4], NULL, 10);
    brc_drop_fn = count_drop;
#if !defined(COUNT_ATOMIC)
    brc_threads = (BrcThread **)calloc((size_t)threads * waves + 4, sizeof(BrcThread *));
#endif
    BrcThread *mainthread = brc_thread_start();
#if !defined(COUNT_ATOMIC)
    brc_threads[mainthread->id] = mainthread;
#endif
    Worker *ws = (Worker *)calloc((size_t)threads, sizeof(Worker));
    HANDLE *hs = (HANDLE *)calloc((size_t)threads, sizeof(HANDLE));
    for (int wv = 0; wv < waves; wv++) {
        for (int i = 0; i < threads; i++) {
            ws[i].rng = seed * 0x9E3779B97F4A7C15ull + (uint64_t)(wv * threads + i) * 0xBF58476D1CE4E5B9ull + 1;
            hs[i] = CreateThread(NULL, 0, worker_main, &ws[i], 0, NULL);
            if (!hs[i]) brc_stop("CreateThread failed");
        }
        WaitForMultipleObjects((DWORD)threads, hs, TRUE, INFINITE);
        for (int i = 0; i < threads; i++) CloseHandle(hs[i]);
    }
    /* Empty every global place (the main thread drops what is left), then merge anything queued to it. */
    for (int g = 0; g < GLOBAL_PLACES; g++) if (places[g].o) { own_dec(mainthread, places[g].o); places[g].o = NULL; }
    brc_thread_end(mainthread);
    long made = brc_made_objects, ended = ended_objects, freed = brc_freed_blocks;
    printf("%s threads=%d waves=%d ops=%ld seed=%llu places=%u made=%ld ended=%ld freed=%ld views=%ld\n",
#if defined(COUNT_ATOMIC)
           "atomic",
#else
           "biased",
#endif
           threads, waves, ops_per_thread, (unsigned long long)seed, nplaces, made, ended, freed, (long)views_checked);
#if !defined(COUNT_ATOMIC)
    printf("paths: queued=%ld closed_drain=%ld merge_at_zero=%ld merge_in_drain=%ld drain_skip_merged=%ld "
           "owner_open=%ld owner_open_dead=%ld shared_open=%ld shared_open_dead=%ld end_after_merge=%ld "
           "end_race_lost=%ld merging_skip=%ld\n", pc_queued, pc_closed_drain, pc_merge_zero, pc_merge_drain,
           pc_merge_skip, pc_owner_open, pc_owner_open_dead, pc_shared_open, pc_shared_open_dead,
           pc_end_after_merge, pc_end_race_lost, pc_merging_skip);
#endif
    if (ended != made) { printf("FAIL: %ld objects never ended (leak)\n", made - ended); return 1; }
    if (freed != made) { printf("FAIL: %ld blocks never freed (leak)\n", made - freed); return 1; }
    printf("OK\n");
    return 0;
}
