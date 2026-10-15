/* The C twin of bench/probes/thread_spawn_join.em: a heap block holding the argument and the
 * result, a thread started on it and joined, with the same checked add. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#else
#include <pthread.h>
#endif
typedef struct { int64_t i; int64_t result; } packet;
#if defined(_WIN32)
static DWORD WINAPI run(LPVOID raw) {
#else
static void* run(void* raw) {
#endif
    packet* p = raw;
    p->result = p->i % 7;
    return 0;
}
int main(void) {
    int64_t total = 0;
    for (int64_t i = 0; i < 20000; ++i) {
        packet* p = malloc(sizeof *p);
        if (!p) abort();
        p->i = i;
#if defined(_WIN32)
        HANDLE h = CreateThread(NULL, 0, run, p, 0, NULL);
        if (!h) abort();
        WaitForSingleObject(h, INFINITE);
        CloseHandle(h);
#else
        pthread_t h;
        if (pthread_create(&h, NULL, run, p)) abort();
        pthread_join(h, NULL);
#endif
        if (__builtin_add_overflow(total, p->result, &total)) abort();
        free(p);
    }
    printf("%lld\n", (long long)total);
    return 0;
}
