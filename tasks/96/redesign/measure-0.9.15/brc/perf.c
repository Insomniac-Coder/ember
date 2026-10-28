/* perf.c: how fast is each way of counting `thread_shared` owners across ordinary usage patterns?
   Build with -DCOUNT_ATOMIC (today's rule), -DCOUNT_BIASED, -DCOUNT_BIASED_CHUNK or -DCOUNT_BIASED_PUB
   (brc.h); -DBRC_COUNT_LOCKS also counts the locked instructions. No checks (blocks freed).

   Usage: perf <pattern> <threads>     prints "<design> <pattern> threads=<n> ms=<wall> ops=<total>"

   Patterns (every thread does the same amount of work; the work grows with the threads):
     own      each thread makes its own objects and copies, drops and opens them (a thread-local cache of
              shared objects, a game's per-thread entity lists): the making thread's own counting
     read     thread 0 made the objects; every thread opens `weakref` links to them and reads (many readers
              of one thread's objects: a shared asset table, a configuration)
     pipe     each thread makes objects and hands one owner each to the next thread through a locked ring,
              which reads it and drops it (a producer/consumer pipeline: drops on another thread)
     fanout   thread 0 makes objects and gives one owner of each to every thread, which keeps it a while
              and drops it (broadcast: copies made on one thread, dropped on all)
     mixed    the stress test's random mix, without its checks */
#include "brc.h"
#include <string.h>

#define NOBJ 4096
#define RING 1024

static int nthreads;
static const char *pattern;
static Obj *table[NOBJ];                 /* "read" and "fanout": objects thread 0 made */
static volatile long go;

/* Each ring starts on its own 128 bytes. */
typedef struct { volatile long lock; unsigned head, tail; char pad[116]; Obj *slot[RING]; } Ring;
static Ring *rings;
static void lockr(Ring *r) { while (_InterlockedExchange(&r->lock, 1)) { while (r->lock) _mm_pause(); } }
static void unlockr(Ring *r) { _InterlockedExchange(&r->lock, 0); }

/* One worker's counters; each worker's record has its own 128 bytes (no two threads share a line). */
typedef struct { int index; uint64_t rng; uint64_t sink; long ops; BrcThread *t; uint64_t cpus; char pad[128 - 6 * 8]; } Worker;
/* The laptop's performance cores (Core Ultra 9 275HX): logical CPUs 0, 1, 10, 11, 12, 13, 22, 23. */
static const int pcores[8] = {0, 1, 10, 11, 12, 13, 22, 23};
static int fixed_cores;                  /* PERF_FIXED set: worker i stays on pcores[i % 8] */
static inline void note_cpu(Worker *w) { w->cpus |= 1ull << GetCurrentProcessorNumber(); }
static inline uint64_t rnd(Worker *w) { uint64_t x = w->rng; x ^= x << 13; x ^= x >> 7; x ^= x << 17; return w->rng = x; }

static void run_own(Worker *w) {
    BrcThread *t = w->t;
    Obj *mine[64], *links[64];
    for (int i = 0; i < 64; i++) { mine[i] = obj_new(t, i); links[i] = mine[i]; weak_add(links[i]); }
    for (long n = 0; n < 4000000; n++) {
        uint64_t r = rnd(w);
        unsigned a = (unsigned)(r & 63), b = (unsigned)((r >> 6) & 63), op = (unsigned)((r >> 12) & 15);
        if (op < 10) {                                        /* open and read: the common case */
            Obj *o = links[a];
            int how = weak_open(t, o);
            if (how) { w->sink += o->value; view_end(t, o, how); }
            else { weak_sub(o); links[a] = mine[a]; weak_add(links[a]); }
        } else if (op < 14) {                                 /* copy an owner into another place */
            if (a != b) { own_inc(t, mine[a]); own_dec(t, mine[b]); mine[b] = mine[a]; }
        } else {                                              /* replace an object */
            Obj *o = obj_new(t, r);
            own_dec(t, mine[a]);
            mine[a] = o;
        }
        w->ops++;
    }
    for (int i = 0; i < 64; i++) { own_dec(t, mine[i]); weak_sub(links[i]); }
}

static void run_read(Worker *w) {
    BrcThread *t = w->t;
    for (long n = 0; n < 8000000; n++) {
        Obj *o = table[rnd(w) & (NOBJ - 1)];
        int how = weak_open(t, o);
        if (how) { w->sink += o->value; view_end(t, o, how); }
        w->ops++;
    }
}

static void run_pipe(Worker *w) {
    BrcThread *t = w->t;
    Ring *in = &rings[w->index], *out = &rings[(w->index + 1) % nthreads];
    long made = 0, taken = 0, want = 2000000;
    while (made < want || taken < want) {
        if (made < want) {                                    /* make one and hand its owner on */
            lockr(out);
            if (out->tail - out->head < RING) {
                Obj *o = obj_new(t, made);
                brc_publish(t, o);                            /* handed to another thread */
                out->slot[out->tail++ % RING] = o;
                made++; w->ops++;
            }
            unlockr(out);
        }
        lockr(in);
        Obj *o = in->tail != in->head ? in->slot[in->head++ % RING] : NULL;
        unlockr(in);
        if (o) {                                              /* read it, keep a link, drop it */
            weak_add(o);
            int how = weak_open(t, o);
            if (how) { w->sink += o->value; view_end(t, o, how); }
            own_dec(t, o);                                    /* a drop of another thread's object */
            weak_sub(o);
            taken++; w->ops++;
        }
        if ((made & 255) == 0) brc_safe_point(t);
    }
    brc_safe_point(t);
}

static void run_fanout(Worker *w) {
    BrcThread *t = w->t;
    Obj *kept[256] = {0};
    for (long n = 0; n < 2000000; n++) {
        unsigned a = (unsigned)(rnd(w) & (NOBJ - 1)), k = (unsigned)(n & 255);
        Obj *o = table[a];
        own_inc(t, o);                                        /* a copy of an owner thread 0 made */
        if (kept[k]) own_dec(t, kept[k]);
        kept[k] = o;
        w->sink += o->value;
        w->ops++;
        if ((n & 255) == 0) brc_safe_point(t);
    }
    for (int k = 0; k < 256; k++) if (kept[k]) own_dec(t, kept[k]);
}

/* The mixed pattern's global places: each place (its lock and its owner) on its own 64 bytes. */
typedef struct { volatile long lock; Obj *volatile o; char pad[64 - 16]; } MixedPlace;
static __declspec(align(64)) MixedPlace mplaces[256];
#define place_locks(g) mplaces[g].lock
#define shared_places(g) mplaces[g].o
static void run_mixed(Worker *w) {
    BrcThread *t = w->t;
    Obj *mine[16] = {0}, *links[16] = {0};
    for (long n = 0; n < 2000000; n++) {
        uint64_t r = rnd(w);
        unsigned op = (unsigned)(r % 100), a = (unsigned)((r >> 8) & 15), b = (unsigned)((r >> 16) & 15);
        unsigned g = (unsigned)((r >> 24) & 255), l = (unsigned)((r >> 32) & 15);
        if (op < 10) { Obj *o = obj_new(t, r); if (mine[a]) own_dec(t, mine[a]); mine[a] = o; }
        else if (op < 18) { if (mine[a] && a != b) { own_inc(t, mine[a]); if (mine[b]) own_dec(t, mine[b]); mine[b] = mine[a]; } }
        else if (op < 26) { if (mine[a]) { own_dec(t, mine[a]); mine[a] = NULL; } }
        else if (op < 36) {
            if (mine[a]) {
                while (_InterlockedExchange(&place_locks(g), 1)) _mm_pause();
                brc_publish(t, mine[a]);
                Obj *old = shared_places(g); shared_places(g) = mine[a];
                _InterlockedExchange(&place_locks(g), 0);
                mine[a] = NULL; if (old) own_dec(t, old);
            }
        } else if (op < 44) {
            while (_InterlockedExchange(&place_locks(g), 1)) _mm_pause();
            Obj *o = shared_places(g); shared_places(g) = NULL;
            _InterlockedExchange(&place_locks(g), 0);
            if (o) { if (mine[a]) own_dec(t, mine[a]); mine[a] = o; }
        } else if (op < 52) {
            while (_InterlockedExchange(&place_locks(g), 1)) _mm_pause();
            Obj *o = shared_places(g); if (o) own_inc(t, o);
            _InterlockedExchange(&place_locks(g), 0);
            if (o) { if (mine[a]) own_dec(t, mine[a]); mine[a] = o; }
        } else if (op < 62) {
            if (mine[a]) { weak_add(mine[a]); if (links[l]) weak_sub(links[l]); links[l] = mine[a]; }
        } else if (op < 97) {
            Obj *o = links[l];
            if (o) { int how = weak_open(t, o); if (how) { w->sink += o->value; view_end(t, o, how); } }
        } else brc_safe_point(t);
        w->ops++;
    }
    for (int i = 0; i < 16; i++) { if (mine[i]) own_dec(t, mine[i]); if (links[i]) weak_sub(links[i]); }
}

static DWORD WINAPI worker_main(LPVOID arg) {
    Worker *w = (Worker *)arg;
    w->t = brc_thread_start();
    if (fixed_cores) SetThreadAffinityMask(GetCurrentThread(), (DWORD_PTR)1 << pcores[w->index % 8]);
    note_cpu(w);
    _InterlockedIncrement(&go);
    while (go < nthreads) _mm_pause();                       /* start together */
    if (!strcmp(pattern, "own")) run_own(w);
    else if (!strcmp(pattern, "read")) run_read(w);
    else if (!strcmp(pattern, "pipe")) run_pipe(w);
    else if (!strcmp(pattern, "fanout")) run_fanout(w);
    else run_mixed(w);
    note_cpu(w);
    brc_thread_end(w->t);
#if defined(BRC_COUNT_LOCKS)
    w->t->locks = brc_locks;
#endif
    return 0;
}

int main(int argc, char **argv) {
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    if (argc != 3) { fprintf(stderr, "usage: perf pattern threads\n"); return 2; }
    pattern = argv[1];
    nthreads = atoi(argv[2]);
    fixed_cores = GetEnvironmentVariableA("PERF_FIXED", NULL, 0) != 0;
    brc_threads = (BrcThread **)calloc((size_t)nthreads + 4, sizeof(BrcThread *));
    BrcThread *mainthread = brc_thread_start();
    /* "read" and "fanout": thread 0's objects are made by the main thread before the workers start
       (so every worker is a reader on another thread's objects). */
    for (int i = 0; i < NOBJ; i++) { table[i] = obj_new(mainthread, (uint64_t)i); brc_publish(mainthread, table[i]); }
    rings = (Ring *)_aligned_malloc((size_t)nthreads * sizeof(Ring), 128);
    Worker *ws = (Worker *)_aligned_malloc((size_t)nthreads * sizeof(Worker), 128);
    if (!rings || !ws) brc_stop("out of memory");
    memset(rings, 0, (size_t)nthreads * sizeof(Ring));
    memset(ws, 0, (size_t)nthreads * sizeof(Worker));
    HANDLE *hs = (HANDLE *)calloc((size_t)nthreads, sizeof(HANDLE));
    LARGE_INTEGER t0, t1, f;
    QueryPerformanceFrequency(&f);
    QueryPerformanceCounter(&t0);
    for (int i = 0; i < nthreads; i++) {
        ws[i].index = i; ws[i].rng = 0x9E3779B97F4A7C15ull * (uint64_t)(i + 1);
        hs[i] = CreateThread(NULL, 0, worker_main, &ws[i], 0, NULL);
    }
    WaitForMultipleObjects((DWORD)nthreads, hs, TRUE, INFINITE);
    QueryPerformanceCounter(&t1);
    long total = 0; uint64_t sink = 0;
    for (int i = 0; i < nthreads; i++) { total += ws[i].ops; sink += ws[i].sink; }
    for (int i = 0; i < NOBJ; i++) own_dec(mainthread, table[i]);
    brc_thread_end(mainthread);
    double ms = (double)(t1.QuadPart - t0.QuadPart) * 1000.0 / (double)f.QuadPart;
    uint64_t cpus = 0;
    for (int i = 0; i < nthreads; i++) cpus |= ws[i].cpus;
    long locks = 0;
#if defined(BRC_COUNT_LOCKS)
    for (int i = 0; i < nthreads; i++) locks += ws[i].t->locks;
#endif
    printf("%s %s threads=%d ms=%.1f ops=%ld sink=%llu cpus=%llx locks_per_op=%.3f\n",
           DESIGN,
           pattern, nthreads, ms, total, (unsigned long long)sink, (unsigned long long)cpus,
           total ? (double)locks / (double)total : 0.0);
    return 0;
}
