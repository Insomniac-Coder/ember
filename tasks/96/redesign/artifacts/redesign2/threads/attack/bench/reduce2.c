/* reduce2.c: where a reduction's accumulator lives decides whether it runs at C speed (A15, A19).
 *
 * Same workload as ../../bench/reduce.c: 64 MiB of text in 8 parts, 8 tasks, `total += 1` for every word.
 *
 *   -DMODEL=0  the accumulator is a C local of the task's function, written to the task record once at the
 *              end (reduce.c model 0; what today's `@parallel(reduce=...)` emits: the chunk function returns
 *              its partial, compiler/ember_codegen_c/src/lib.rs `emit_parallel_helpers`). Possible only when
 *              every `total += e` is written in the task's own body.
 *   -DMODEL=3  the accumulator lives in the task record and is reached through a pointer, as it must be when
 *              the `+=` is inside a function the task calls (`fn add_words(t): global WORDS; WORDS += ...`):
 *              one load-add-store per word (the text's byte loads may alias it, so clang keeps it in memory).
 *              The 8 records are adjacent (24 bytes each), as task records allocated together are.
 *   -DMODEL=4  as 3, each record padded to its own 64-byte cache line
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n reduce2.c -o bin/reduce2_n.exe */
#include "../../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#define TEXT_BYTES (64u << 20)
#define TASKS 8
#define REPEAT 4

static uint8_t *text;

#if MODEL == 4
typedef struct { const uint8_t *p; size_t n; uint64_t local; char pad[40]; } Task;   /* 64 bytes */
#else
typedef struct { const uint8_t *p; size_t n; uint64_t local; } Task;                /* 24 bytes */
#endif

#if MODEL == 3 || MODEL == 4
/* The called function: it adds to the accumulator it is handed. Not inlined, as a function in another
 * module would not be; its loop is the word scan. */
static __declspec(noinline) void add_words(const uint8_t *p, size_t n, uint64_t *acc) {
    for (size_t i = 0; i + 1 < n; i++)
        if (p[i] != ' ' && p[i + 1] == ' ') *acc += 1;
}
#endif

static DWORD WINAPI work(LPVOID arg) {
    Task *t = (Task *)arg;
#if MODEL == 0
    const uint8_t *p = t->p;
    uint64_t local = 0;
    for (int r = 0; r < REPEAT; r++)
        for (size_t i = 0; i + 1 < t->n; i++)
            if (p[i] != ' ' && p[i + 1] == ' ') local += 1;
    t->local = local;
#else
    for (int r = 0; r < REPEAT; r++) add_words(t->p, t->n, &t->local);
#endif
    return 0;
}

int main(void) {
    h_start();
    h_seed(7);
    text = (uint8_t *)malloc(TEXT_BYTES);
    if (!text) abort();
    for (size_t i = 0; i < TEXT_BYTES; i++) text[i] = (h_rand() % 6 == 0) ? ' ' : (uint8_t)('a' + h_rand() % 26);
    for (int k = 1; k <= TASKS; k++) text[(size_t)k * (TEXT_BYTES / TASKS) - 1] = ' ';
    h_start();
    static __declspec(align(64)) Task tasks[TASKS];
    HANDLE th[TASKS];
    for (int k = 0; k < TASKS; k++) {
        tasks[k].p = text + (size_t)k * (TEXT_BYTES / TASKS);
        tasks[k].n = TEXT_BYTES / TASKS;
        tasks[k].local = 0;
        th[k] = CreateThread(NULL, 0, work, &tasks[k], 0, NULL);
        if (!th[k]) abort();
    }
    WaitForMultipleObjects(TASKS, th, TRUE, INFINITE);
    uint64_t total = 0;
    for (int k = 0; k < TASKS; k++) total += tasks[k].local;
    static const char *names[] = {"reduce2-0-local-accumulator", "", "", "reduce2-3-through-pointer-packed",
                                  "reduce2-4-through-pointer-padded"};
    h_report(names[MODEL], total);
    return 0;
}
