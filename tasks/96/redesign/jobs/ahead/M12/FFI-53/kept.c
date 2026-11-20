#include "kept.h"
#include <stddef.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <pthread.h>
#endif

static kept_fn kept_cb = NULL;
static void* kept_data = NULL;

void keep(kept_fn cb, void* data) {
    kept_cb = cb;
    kept_data = data;
}

void keep_once(kept_fn cb, void* data) {
    kept_cb = cb;
    kept_data = data;
}

void keep_here(kept_fn cb, void* data) {
    kept_cb = cb;
    kept_data = data;
}

void unkeep(kept_fn cb, void* data) {
    (void)cb;
    (void)data;
}

int fire(int value) {
    return kept_cb != NULL ? kept_cb(kept_data, value) : -1;
}

struct call { int value; int result; };

#ifdef _WIN32
static DWORD WINAPI run(LPVOID argument) {
    struct call* call = (struct call*)argument;
    call->result = fire(call->value);
    return 0;
}
#else
static void* run(void* argument) {
    struct call* call = (struct call*)argument;
    call->result = fire(call->value);
    return NULL;
}
#endif

int fire_elsewhere(int value) {
    struct call call = { value, -1 };
#ifdef _WIN32
    HANDLE thread = CreateThread(NULL, 0, run, &call, 0, NULL);
    WaitForSingleObject(thread, INFINITE);
    CloseHandle(thread);
#else
    pthread_t thread;
    pthread_create(&thread, NULL, run, &call);
    pthread_join(thread, NULL);
#endif
    return call.result;
}

int tagged(void) {
    if (sizeof(void*) < 8) {
        return 1;
    }
    uint64_t u = (uint64_t)(uintptr_t)kept_data;
    return (int)(((u >> 48) & 1u) && ((u >> 55) & 1u) && ((u >> 63) & 1u));
}
