/* The C twin of bench/probes/channel_throughput.em: a 64-slot ring buffer under the platform's
 * lock, a condition variable for "not full" and one for "not empty", one producer thread, and the
 * consumer adding with the same overflow check. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
typedef SRWLOCK lock_t;
typedef CONDITION_VARIABLE cond_t;
#define LOCK(l) AcquireSRWLockExclusive(l)
#define UNLOCK(l) ReleaseSRWLockExclusive(l)
#define WAIT(c, l) SleepConditionVariableSRW(c, l, INFINITE, 0)
#define SIGNAL(c) WakeConditionVariable(c)
#else
#include <pthread.h>
typedef pthread_mutex_t lock_t;
typedef pthread_cond_t cond_t;
#define LOCK(l) pthread_mutex_lock(l)
#define UNLOCK(l) pthread_mutex_unlock(l)
#define WAIT(c, l) pthread_cond_wait(c, l)
#define SIGNAL(c) pthread_cond_signal(c)
#endif
#define CAP 64
static struct {
    lock_t lock;
    cond_t not_full, not_empty;
    int64_t slots[CAP];
    int head, count;
    bool closed;
} q;
static int64_t n = 1000000;
#if defined(_WIN32)
static DWORD WINAPI produce(LPVOID unused) {
#else
static void* produce(void* unused) {
#endif
    (void)unused;
    for (int64_t i = 0; i < n; ++i) {
        LOCK(&q.lock);
        while (q.count == CAP) WAIT(&q.not_full, &q.lock);
        q.slots[(q.head + q.count) % CAP] = i % 1000;
        q.count += 1;
        SIGNAL(&q.not_empty);
        UNLOCK(&q.lock);
    }
    LOCK(&q.lock);
    q.closed = true;
    SIGNAL(&q.not_empty);
    UNLOCK(&q.lock);
    return 0;
}
int main(void) {
#if defined(_WIN32)
    InitializeSRWLock(&q.lock);
    InitializeConditionVariable(&q.not_full);
    InitializeConditionVariable(&q.not_empty);
    HANDLE t = CreateThread(NULL, 0, produce, NULL, 0, NULL);
#else
    pthread_mutex_init(&q.lock, NULL);
    pthread_cond_init(&q.not_full, NULL);
    pthread_cond_init(&q.not_empty, NULL);
    pthread_t t;
    pthread_create(&t, NULL, produce, NULL);
#endif
    int64_t total = 0;
    for (;;) {
        LOCK(&q.lock);
        while (q.count == 0 && !q.closed) WAIT(&q.not_empty, &q.lock);
        if (q.count == 0) { UNLOCK(&q.lock); break; }
        int64_t v = q.slots[q.head];
        q.head = (q.head + 1) % CAP;
        q.count -= 1;
        SIGNAL(&q.not_full);
        UNLOCK(&q.lock);
        if (__builtin_add_overflow(total, v, &total)) abort();
    }
#if defined(_WIN32)
    WaitForSingleObject(t, INFINITE);
#else
    pthread_join(t, NULL);
#endif
    printf("%lld\n", (long long)total);
    return 0;
}
