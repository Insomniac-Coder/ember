/* cb_once_lib.h: workload cb_once's C library, shared by every C program of the workload: a queue
   of requests; `lib_pump` runs each queued callback once and empties the queue. Not inlined; the
   callback pointers it holds come from the caller's volatile, so the C compiler cannot see them. */
#pragma once
#include "m4.h"

#define BATCH 1024
#define BATCHES 48828

typedef void (*cb_t)(void *);
static struct { cb_t cb; void *ud; } lib_queue[BATCH];
static int lib_n;

static NOINLINE void lib_request(cb_t cb, void *ud) {
    lib_queue[lib_n].cb = cb; lib_queue[lib_n].ud = ud; lib_n++;
}
static NOINLINE void lib_pump(void) {
    int n = lib_n;
    lib_n = 0;
    for (int i = 0; i < n; i++) lib_queue[i].cb(lib_queue[i].ud);
}
