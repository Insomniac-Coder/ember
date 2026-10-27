/* harness.h: shared by every C model in target/redesign/bench (Windows x64, clang -O2).
   Call h_start() first thing in main and h_report() last: it prints one line,
   "<name> ms=<wall ms> peak_kb=<peak working set KB> checksum=<u64>". */
#pragma once
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <psapi.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#pragma comment(lib, "psapi")

/* The one random source every workload uses, so every model does the same work. */
static uint64_t h_rng = 1;
static inline void h_seed(uint64_t s) { h_rng = s; }
static inline uint32_t h_rand(void) {
    h_rng = h_rng * 6364136223846793005ull + 1442695040888963407ull;
    return (uint32_t)(h_rng >> 33);
}

static LARGE_INTEGER h_t0;
static inline void h_start(void) {
    /* No error dialogs on the owner's screen if a model crashes. */
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    QueryPerformanceCounter(&h_t0);
}
static inline void h_report(const char *name, uint64_t checksum) {
    LARGE_INTEGER t1, f;
    QueryPerformanceCounter(&t1);
    QueryPerformanceFrequency(&f);
    PROCESS_MEMORY_COUNTERS pmc;
    GetProcessMemoryInfo(GetCurrentProcess(), &pmc, sizeof pmc);
    printf("%s ms=%.3f peak_kb=%llu checksum=%llu\n", name,
           (double)(t1.QuadPart - h_t0.QuadPart) * 1000.0 / (double)f.QuadPart,
           (unsigned long long)(pmc.PeakWorkingSetSize / 1024), (unsigned long long)checksum);
}
