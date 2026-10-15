/* The C twin of bench/probes/mutex_contended.em: two threads each taking the platform's lock two
 * million times to add one under it, with the same checked add. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
static SRWLOCK lock = SRWLOCK_INIT;
#define LOCK() AcquireSRWLockExclusive(&lock)
#define UNLOCK() ReleaseSRWLockExclusive(&lock)
#else
#include <pthread.h>
static pthread_mutex_t lock = PTHREAD_MUTEX_INITIALIZER;
#define LOCK() pthread_mutex_lock(&lock)
#define UNLOCK() pthread_mutex_unlock(&lock)
#endif
static int64_t value;
static int64_t n = 2000000;
#if defined(_WIN32)
static DWORD WINAPI bump(LPVOID unused) {
#else
static void* bump(void* unused) {
#endif
    (void)unused;
    for (int64_t i = 0; i < n; ++i) {
        LOCK();
        if (__builtin_add_overflow(value, 1, &value)) abort();
        UNLOCK();
    }
    return 0;
}
int main(void) {
#if defined(_WIN32)
    HANDLE t = CreateThread(NULL, 0, bump, NULL, 0, NULL);
    bump(NULL);
    WaitForSingleObject(t, INFINITE);
#else
    pthread_t t;
    pthread_create(&t, NULL, bump, NULL);
    bump(NULL);
    pthread_join(t, NULL);
#endif
    printf("%lld\n", (long long)value);
    return 0;
}
