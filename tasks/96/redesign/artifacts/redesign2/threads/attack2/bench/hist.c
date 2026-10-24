/* hist.c: an element reduction (probes/y21's `hist[b] += 1` written in tasks), against what else could be
 * emitted for it. 32 MiB of random bytes, 8 tasks, a 256-entry histogram.
 *
 *   -DMODEL=0  plain C on one thread (what the program computes)
 *   -DMODEL=1  the proposed element reduction: each task fills its own 256-entry histogram (a local array),
 *              and the parent adds the 8 histograms at the join, in spawn order. This is what a C programmer
 *              writes by hand (and OpenMP's `reduction(+: hist[:256])`)
 *   -DMODEL=2  one shared histogram, an atomic add per byte (today's spelling with Atomic elements)
 *   -DMODEL=3  one shared histogram under one lock, taken per byte (Python's `with lock: hist[b] += 1`)
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n hist.c -o bin/hist_n.exe */
#include "../../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#define SIZE (32u << 20)
#define TASKS 8

static uint8_t *data;
static int64_t hist[256];
static SRWLOCK lock = SRWLOCK_INIT;
typedef struct { size_t lo, hi; int64_t mine[256]; } Part;

static DWORD WINAPI work(LPVOID arg) {
    Part *pt = (Part *)arg;
    const uint8_t *d = data;
#if MODEL == 1
    int64_t h[256] = {0};
    for (size_t i = pt->lo; i < pt->hi; i++) h[d[i]] += 1;
    for (int b = 0; b < 256; b++) pt->mine[b] = h[b];
#elif MODEL == 2
    for (size_t i = pt->lo; i < pt->hi; i++) __atomic_fetch_add(&hist[d[i]], 1, __ATOMIC_RELAXED);
#elif MODEL == 3
    for (size_t i = pt->lo; i < pt->hi; i++) { AcquireSRWLockExclusive(&lock); hist[d[i]] += 1; ReleaseSRWLockExclusive(&lock); }
#endif
    return 0;
}

int main(void) {
    h_start();
    h_seed(5);
    data = (uint8_t *)malloc(SIZE);
    if (!data) abort();
    for (size_t i = 0; i < SIZE; i++) data[i] = (uint8_t)(h_rand() >> 7);
    h_start();
#if MODEL == 0
    for (size_t i = 0; i < SIZE; i++) hist[data[i]] += 1;
#else
    static Part parts[TASKS];
    HANDLE th[TASKS];
    for (int k = 0; k < TASKS; k++) {
        parts[k].lo = (size_t)k * (SIZE / TASKS); parts[k].hi = parts[k].lo + SIZE / TASKS;
        th[k] = CreateThread(NULL, 0, work, &parts[k], 0, NULL);
        if (!th[k]) abort();
    }
    WaitForMultipleObjects(TASKS, th, TRUE, INFINITE);
    for (int k = 0; k < TASKS; k++) CloseHandle(th[k]);
#if MODEL == 1
    for (int k = 0; k < TASKS; k++) for (int b = 0; b < 256; b++) hist[b] += parts[k].mine[b];
#endif
#endif
    uint64_t sum = 0;
    for (int b = 0; b < 256; b++) sum = sum * 31 + (uint64_t)hist[b];
    static const char *names[] = {"hist-0-plainC-one-thread", "hist-1-element-reduction", "hist-2-shared-atomic", "hist-3-shared-lock"};
    h_report(names[MODEL], sum);
    return 0;
}
