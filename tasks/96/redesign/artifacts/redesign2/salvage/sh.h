/* sh.h: the shared harness (target/redesign/bench/harness.h) plus an optional phase timer.
   H_PHASE("name") prints the time since h_start() to stderr only when built with -DPHASES;
   the timed builds are made without it. */
#pragma once
#include "harness.h"
#ifdef PHASES
#define H_PHASE(name)                                                                               \
    do {                                                                                            \
        LARGE_INTEGER a_, f_;                                                                       \
        QueryPerformanceCounter(&a_);                                                               \
        QueryPerformanceFrequency(&f_);                                                             \
        fprintf(stderr, "phase %s %.3f ms\n", name,                                                 \
                (double)(a_.QuadPart - h_t0.QuadPart) * 1000.0 / (double)f_.QuadPart);              \
    } while (0)
#else
#define H_PHASE(name) ((void)0)
#endif
