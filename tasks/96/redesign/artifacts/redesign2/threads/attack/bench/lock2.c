/* lock2.c: ../../bench/lock.c with the use made longer (-DREPS, hashes per call; lock.c has 4) and
 * -DCALLS per thread, to see when holding the lock for the caller's use (the accessor, model 0) stops
 * beating a copy out of the lock (model 1). Everything else is lock.c unchanged (generated with sed from it).
 * Build: clang -O2 -DMODEL=n -DREPS=r -DCALLS=c -DNAME0=... -DNAME1=... lock2.c */
/* lock.c: 8 threads read the top name of a locked score table, one of them sometimes replaces it (A33).
 *
 * The table is `Mutex[Array[String]]` (Python: a list behind `threading.Lock`). Each thread calls
 * `top()` 2,000,000 times and uses the name (hashes its bytes 4 times, outside the lock where the model
 * allows). Thread 0 replaces `scores[0]` with a new string every 1024th call (free the old, make a new
 * one, under the lock). Names are 16 to 23 bytes. All models do the same calls, same checksum.
 *
 *   -DMODEL=0  the name is used inside the lock (lock, hash in place, unlock): the proposed accessor
 *              (`with name = board.top():`), and what a C programmer writes when the use is short
 *   -DMODEL=1  a copy of the String leaves the lock (lock, malloc and copy, unlock, hash, free): the
 *              proposed `return g[0]` (a copy, as Rust's `.clone()`)
 *   -DMODEL=2  the copy goes to a stack buffer instead of the heap (what a C programmer writes when the
 *              size is small and known; Ember gets this only if strings have an inline small form)
 *   -DMODEL=3  snapshots: readers take the current version with a shared lock and an atomic count
 *              increment, use it with no lock, and decrement; the writer copies the table, swaps the
 *              pointer under the exclusive lock, and the last reader of the old version frees it
 * The lock is a Win32 SRW lock in every model.
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n lock.c -o lock_n.exe */
#include "../../../../redesign/bench/harness.h"
#include <string.h>

#ifndef MODEL
#define MODEL 0
#endif
#define THREADS 8
#ifndef CALLS
#define CALLS 2000000
#endif
#ifndef REPS
#define REPS 4
#endif
#define NAMES 64

typedef struct { char *p; size_t len; } Str;
static Str make_name(uint64_t k) {
    char buf[32];
    int n = snprintf(buf, sizeof buf, "player-%llu", (unsigned long long)(k * 2654435761ull));
    Str s = { (char *)malloc((size_t)n), (size_t)n };
    if (!s.p) abort();
    memcpy(s.p, buf, (size_t)n);
    return s;
}
static inline uint64_t use_name(const char *p, size_t n) {
    uint64_t h = 1469598103934665603ull;
    for (int r = 0; r < REPS; r++)
        for (size_t i = 0; i < n; i++) h = (h ^ (uint8_t)p[i]) * 1099511628211ull;
    return h;
}

static SRWLOCK lock = SRWLOCK_INIT;
#if MODEL == 3
typedef struct { volatile LONG count; Str names[NAMES]; } Version;
static Version *current;
static void version_release(Version *v) {
    if (InterlockedDecrement(&v->count) == 0) {
        for (int i = 0; i < NAMES; i++) free(v->names[i].p);
        free(v);
    }
}
#else
static Str scores[NAMES];
#endif

static DWORD WINAPI reader(LPVOID arg) {
    uint64_t id = (uint64_t)(uintptr_t)arg, acc = 0, next = 1000000 * (id + 1);
    for (int c = 0; c < CALLS; c++) {
        if (id == 0 && (c & 1023) == 0) {                  /* the writer: scores[0] = new name */
            Str fresh = make_name(next++);
#if MODEL == 3
            Version *nv = (Version *)malloc(sizeof(Version));
            if (!nv) abort();
            AcquireSRWLockShared(&lock);
            Version *old = current;
            InterlockedIncrement(&old->count);
            ReleaseSRWLockShared(&lock);
            nv->count = 1;
            nv->names[0] = fresh;
            for (int i = 1; i < NAMES; i++) {                  /* copy the rest of the table */
                size_t n = old->names[i].len;
                nv->names[i] = (Str){ (char *)malloc(n), n };
                if (!nv->names[i].p) abort();
                memcpy(nv->names[i].p, old->names[i].p, n);
            }
            AcquireSRWLockExclusive(&lock);
            current = nv;
            ReleaseSRWLockExclusive(&lock);
            version_release(old);                          /* the table's reference */
            version_release(old);                          /* the writer's own */
#else
            AcquireSRWLockExclusive(&lock);
            free(scores[0].p);
            scores[0] = fresh;
            ReleaseSRWLockExclusive(&lock);
#endif
        }
#if MODEL == 0
        AcquireSRWLockExclusive(&lock);
        acc += use_name(scores[0].p, scores[0].len);
        ReleaseSRWLockExclusive(&lock);
#elif MODEL == 1
        AcquireSRWLockExclusive(&lock);
        Str copy = { (char *)malloc(scores[0].len), scores[0].len };
        if (!copy.p) abort();
        memcpy(copy.p, scores[0].p, copy.len);
        ReleaseSRWLockExclusive(&lock);
        acc += use_name(copy.p, copy.len);
        free(copy.p);
#elif MODEL == 2
        char buf[32];
        AcquireSRWLockExclusive(&lock);
        size_t n = scores[0].len;
        memcpy(buf, scores[0].p, n);
        ReleaseSRWLockExclusive(&lock);
        acc += use_name(buf, n);
#else
        AcquireSRWLockShared(&lock);
        Version *v = current;
        InterlockedIncrement(&v->count);
        ReleaseSRWLockShared(&lock);
        acc += use_name(v->names[0].p, v->names[0].len);
        version_release(v);
#endif
    }
    return (DWORD)(acc & 0x7FFFFFFF);
}

int main(void) {
    h_start();
#if MODEL == 3
    current = (Version *)malloc(sizeof(Version));
    if (!current) abort();
    current->count = 1;
    for (int i = 0; i < NAMES; i++) current->names[i] = make_name((uint64_t)i);
#else
    for (int i = 0; i < NAMES; i++) scores[i] = make_name((uint64_t)i);
#endif
    HANDLE th[THREADS];
    for (int k = 0; k < THREADS; k++) {
        th[k] = CreateThread(NULL, 0, reader, (LPVOID)(uintptr_t)k, 0, NULL);
        if (!th[k]) abort();
    }
    WaitForMultipleObjects(THREADS, th, TRUE, INFINITE);
    /* The checksum is the final top name's hash: the same in every model (the writer's sequence is fixed). */
#if MODEL == 3
    uint64_t sum = use_name(current->names[0].p, current->names[0].len);
#else
    uint64_t sum = use_name(scores[0].p, scores[0].len);
#endif
    static const char *names[] = {NAME0, NAME1, "lock-2-copy-out-stack",
                                  "lock-3-snapshot"};
    h_report(names[MODEL], sum);
    return 0;
}
