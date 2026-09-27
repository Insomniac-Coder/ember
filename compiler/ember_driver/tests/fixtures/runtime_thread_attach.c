#include "../../../../runtime/ember_rt/src/ember_rt.c"

#if defined(_WIN32)
#include <windows.h>
#define WORKER_RESULT(code) ((DWORD)(code))
#define PAUSE_THREAD() Sleep(1)
static volatile LONG stage;
static void set_stage(int value) { InterlockedExchange(&stage, value); }
static int get_stage(void) { return (int)InterlockedCompareExchange(&stage, 0, 0); }
static DWORD WINAPI check_worker(LPVOID unused) {
    (void)unused;
#else
#include <pthread.h>
#include <sched.h>
#include <stdatomic.h>
#define WORKER_RESULT(code) ((void*)(uintptr_t)(code))
#define PAUSE_THREAD() sched_yield()
static _Atomic int stage;
static void set_stage(int value) { atomic_store(&stage, value); }
static int get_stage(void) { return atomic_load(&stage); }
static void* check_worker(void* unused) {
    (void)unused;
#endif
    if (runtime_thread_attached()) return WORKER_RESULT(1);
    ember_rt_thread_attach();
    if (!runtime_thread_attached()) return WORKER_RESULT(2);
    ember_rt_thread_attach();
    if (!runtime_thread_attached()) return WORKER_RESULT(3);
    ember_rt_thread_detach();
    if (runtime_thread_attached()) return WORKER_RESULT(4);
    return WORKER_RESULT(0);
}

#if defined(_WIN32)
static DWORD WINAPI check_shutdown_worker(LPVOID unused) {
#else
static void* check_shutdown_worker(void* unused) {
#endif
    (void)unused;
    ember_rt_thread_attach();
    if (!runtime_thread_attached()) return WORKER_RESULT(6);
    set_stage(1);
    for (int tries = 0; tries < 100000 && get_stage() != 2; ++tries) PAUSE_THREAD();
    if (get_stage() != 2) return WORKER_RESULT(7);
    return WORKER_RESULT(runtime_thread_attached() ? 8 : 0);
}

int main(void) {
    if (runtime_thread_attached()) return 1;
    ember_rt_thread_attach();
    if (!runtime_thread_attached()) return 2;
#if defined(_WIN32)
    HANDLE thread = CreateThread(NULL, 0, check_worker, NULL, 0, NULL);
    if (thread == NULL) return 3;
    WaitForSingleObject(thread, INFINITE);
    DWORD result = 0;
    if (!GetExitCodeThread(thread, &result)) return 4;
    CloseHandle(thread);
    if (result != 0) return 5;
#else
    pthread_t thread;
    if (pthread_create(&thread, NULL, check_worker, NULL) != 0) return 3;
    void* result = NULL;
    if (pthread_join(thread, &result) != 0) return 4;
    if (result != NULL) return 5;
#endif
    if (!runtime_thread_attached()) return 6;
    ember_rt_thread_detach();
    if (runtime_thread_attached()) return 7;

    ember_rt_init(NULL);
#if defined(_WIN32)
    thread = CreateThread(NULL, 0, check_shutdown_worker, NULL, 0, NULL);
    if (thread == NULL) return 8;
#else
    if (pthread_create(&thread, NULL, check_shutdown_worker, NULL) != 0) return 8;
#endif
    for (int tries = 0; tries < 100000 && get_stage() != 1; ++tries) PAUSE_THREAD();
    if (get_stage() != 1) return 9;
    ember_rt_shutdown();
    set_stage(2);
#if defined(_WIN32)
    WaitForSingleObject(thread, INFINITE);
    if (!GetExitCodeThread(thread, &result)) return 10;
    CloseHandle(thread);
    if (result != 0) return 11;
#else
    if (pthread_join(thread, &result) != 0) return 10;
    if (result != NULL) return 11;
#endif
    if (runtime_thread_attached()) return 12;
    return 0;
}
