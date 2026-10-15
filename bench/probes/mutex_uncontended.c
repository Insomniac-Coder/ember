/* The C twin of bench/probes/mutex_uncontended.em: the platform's own lock taken and released
 * around a checked add, as Ember computes it. */
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
int main(void) {
    int64_t n = 50000000;
    for (int64_t i = 0; i < n; ++i) {
        LOCK();
        if (__builtin_add_overflow(value, i % 7, &value)) abort();
        UNLOCK();
    }
    LOCK();
    printf("%lld\n", (long long)value);
    UNLOCK();
    return 0;
}
