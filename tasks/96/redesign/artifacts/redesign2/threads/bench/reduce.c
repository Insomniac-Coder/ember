/* reduce.c: `total += 1` for every word, from 8 scoped tasks (A15 and A19: a reduction against the
 * spellings that keep one shared counter).
 *
 * 64 MiB of random text (letters and spaces, a word ends at a space), cut into 8 equal parts, one task
 * per part. Every task walks its part and adds 1 to the total for every word it finds. Same text,
 * same count in every model. Threads are Win32 threads; the scope's join is WaitForMultipleObjects.
 *
 *   -DMODEL=0  plain C as a C programmer writes it: a local counter per thread, added after the join.
 *              This is also exactly what the proposed reduction compiles to (one accumulator in each
 *              task's state, combined in spawn order at the scope's end), so there is no separate model.
 *   -DMODEL=1  one shared atomic counter, one atomic add per word: the spelling today's Ember needs
 *              (`total.fetch_add(1)`, `[THR-14]`, sequentially consistent; on x86 `lock xadd`, which a
 *              relaxed add also compiles to)
 *   -DMODEL=2  one shared counter under a lock, one lock and unlock per word: the literal translation of
 *              Python's `with lock: total += 1` (an SRW lock, which Ember's `Mutex` uses on Windows)
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n reduce.c -o reduce_n.exe */
#include "../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#define TEXT_BYTES (64u << 20)
#define TASKS 8
#define REPEAT 4

static uint8_t *text;
static volatile LONG64 shared_total;     /* models 1 and 2 */
static SRWLOCK lock = SRWLOCK_INIT;

typedef struct { const uint8_t *p; size_t n; uint64_t local; } Task;

static DWORD WINAPI work(LPVOID arg) {
    Task *t = (Task *)arg;
    const uint8_t *p = t->p;
    uint64_t local = 0;                   /* model 0: the task's own accumulator */
    for (int r = 0; r < REPEAT; r++) {
        for (size_t i = 0; i + 1 < t->n; i++) {
            if (p[i] != ' ' && p[i + 1] == ' ') {          /* a word ends here */
#if MODEL == 0
                local += 1;
#elif MODEL == 1
                __atomic_fetch_add(&shared_total, 1, __ATOMIC_SEQ_CST);
#else
                AcquireSRWLockExclusive(&lock);
                shared_total += 1;
                ReleaseSRWLockExclusive(&lock);
#endif
            }
        }
    }
    t->local = local;
    return 0;
}

int main(void) {
    h_start();
    h_seed(7);
    text = (uint8_t *)malloc(TEXT_BYTES);
    if (!text) abort();
    for (size_t i = 0; i < TEXT_BYTES; i++) text[i] = (h_rand() % 6 == 0) ? ' ' : (uint8_t)('a' + h_rand() % 26);
    for (int k = 1; k <= TASKS; k++) text[(size_t)k * (TEXT_BYTES / TASKS) - 1] = ' ';   /* parts end at a space */
    h_start();                            /* time the scope only, not making the text */
    Task tasks[TASKS];
    HANDLE th[TASKS];
    for (int k = 0; k < TASKS; k++) {
        tasks[k].p = text + (size_t)k * (TEXT_BYTES / TASKS);
        tasks[k].n = TEXT_BYTES / TASKS;
        tasks[k].local = 0;
        th[k] = CreateThread(NULL, 0, work, &tasks[k], 0, NULL);
        if (!th[k]) abort();
    }
    WaitForMultipleObjects(TASKS, th, TRUE, INFINITE);    /* the scope's end */
    uint64_t total = 0;
#if MODEL == 0
    for (int k = 0; k < TASKS; k++) total += tasks[k].local;   /* combined in spawn order */
#else
    total = (uint64_t)shared_total;
#endif
    static const char *names[] = {"reduce-0-plainC-and-reduction", "reduce-1-atomic-per-word", "reduce-2-lock-per-word"};
    h_report(names[MODEL], total);
    return 0;
}
