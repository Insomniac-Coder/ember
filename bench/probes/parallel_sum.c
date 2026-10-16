/* Twin of parallel_sum.em: the same sum split by hand into one even piece per logical processor,
   one thread each (the main thread takes the first), joined and added in order. */
#include <stdint.h>
#include <stdio.h>
#include <windows.h>

#define N 400000000LL

typedef struct piece {
    int64_t lo;
    int64_t hi;
    int64_t sum;
} piece;

static DWORD WINAPI run(LPVOID data) {
    piece* p = (piece*)data;
    int64_t sum = 0;
    for (int64_t i = p->lo; i < p->hi; ++i) {
        sum += (i * 2654435761LL) % 1000;
    }
    p->sum = sum;
    return 0;
}

int main(void) {
    DWORD count = GetActiveProcessorCount(ALL_PROCESSOR_GROUPS);
    if (count < 1) count = 1;
    if (count > 64) count = 64;
    piece pieces[64];
    HANDLE threads[64];
    int64_t step = (N + count - 1) / count;
    for (DWORD t = 0; t < count; ++t) {
        pieces[t].lo = (int64_t)t * step;
        pieces[t].hi = pieces[t].lo + step < N ? pieces[t].lo + step : N;
        pieces[t].sum = 0;
    }
    for (DWORD t = 1; t < count; ++t) {
        threads[t] = CreateThread(NULL, 0, run, &pieces[t], 0, NULL);
        if (threads[t] == NULL) return 1;
    }
    run(&pieces[0]);
    int64_t total = pieces[0].sum;
    for (DWORD t = 1; t < count; ++t) {
        WaitForSingleObject(threads[t], INFINITE);
        CloseHandle(threads[t]);
        total += pieces[t].sum;
    }
    printf("%lld\n", (long long)total);
    return 0;
}
