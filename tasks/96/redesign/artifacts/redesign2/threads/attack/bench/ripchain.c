/* ripchain.c: why did ../../bench/globals.c's model 4 (a base pointer) run 0.55x of model 0 (plain globals)?
 *
 * Hypothesis: the CPU passes a value from a store to a later load of the same address with almost no delay
 * when both name the address the same way through a base register, but not when both address it from the
 * instruction pointer (RIP-relative), where each instruction's displacement differs. A function that adds 1
 * to a global on every call makes a chain of store -> load through memory, so that delay is the whole cost.
 *
 *   -DMODEL=0  a loop of `G += 1` on a volatile global, addressed from the instruction pointer
 *   -DMODEL=1  the same through a pointer held in a register (loaded once before the loop)
 *   -DMODEL=2  `tick()` (not inlined) does `G += 1` on a plain global (RIP-relative), called in a loop
 *   -DMODEL=3  `tick()` does the same through a base pointer it loads at entry (globals.c model 4's form)
 *   -DMODEL=4  as 2, but tick() also does unrelated work between calls (a 16-step multiply chain on a local),
 *              so the store -> load chain is not the only thing on the critical path
 *   -DMODEL=5  as 3 with that work
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n ripchain.c -o bin/ripchain_n.exe */
#include "../../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#define ITERS 1000000000ull
#define CALLS 300000000u

static volatile int64_t VG;
static int64_t G;
static int64_t *volatile G_base = &G;
static uint64_t work_seed = 12345;

#if MODEL >= 2
static __declspec(noinline) void tick(void) {
#if MODEL == 3 || MODEL == 5
    int64_t *b = G_base;
#define TG (*b)
#else
#define TG G
#endif
    TG += 1;
#if MODEL >= 4
    uint64_t x = work_seed;
    for (int k = 0; k < 16; k++) x = x * 6364136223846793005ull + 1;
    work_seed = x;
#endif
}
#endif

int main(void) {
    h_start();
    uint64_t sum;
#if MODEL == 0
    for (uint64_t i = 0; i < ITERS; i++) VG = VG + 1;
    sum = (uint64_t)VG;
#elif MODEL == 1
    volatile int64_t *p = (volatile int64_t *)G_base;
    for (uint64_t i = 0; i < ITERS; i++) *p = *p + 1;
    sum = (uint64_t)*p;
#else
    for (uint32_t i = 0; i < CALLS; i++) tick();
    sum = (uint64_t)G + work_seed;
#endif
    static const char *names[] = {"ripchain-0-loop-rip", "ripchain-1-loop-base", "ripchain-2-call-rip",
                                  "ripchain-3-call-base", "ripchain-4-call-rip-work", "ripchain-5-call-base-work"};
    h_report(names[MODEL], sum);
    return 0;
}
