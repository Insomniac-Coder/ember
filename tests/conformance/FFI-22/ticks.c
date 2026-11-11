#include "ticks.h"
#ifdef _WIN32
#include <windows.h>
#else
#include <pthread.h>
#endif

void tick_here(tick_fn tick, void* data) {
    tick(data);
}

struct call { tick_fn tick; void* data; };

#ifdef _WIN32
static DWORD WINAPI run(LPVOID argument) {
    struct call* call = (struct call*)argument;
    call->tick(call->data);
    return 0;
}
#else
static void* run(void* argument) {
    struct call* call = (struct call*)argument;
    call->tick(call->data);
    return NULL;
}
#endif

void tick_elsewhere(tick_fn tick, void* data) {
    struct call call = { tick, data };
#ifdef _WIN32
    HANDLE thread = CreateThread(NULL, 0, run, &call, 0, NULL);
    WaitForSingleObject(thread, INFINITE);
    CloseHandle(thread);
#else
    pthread_t thread;
    pthread_create(&thread, NULL, run, &call);
    pthread_join(thread, NULL);
#endif
}
