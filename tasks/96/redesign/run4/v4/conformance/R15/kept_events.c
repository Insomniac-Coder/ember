#include "kept_events.h"
#ifdef _WIN32
#include <windows.h>
#else
#include <pthread.h>
#endif

static ke_fn kept_cb;
static void* kept_ud;
static ke_fn stale_cb;
static void* stale_ud;

void ke_register(ke_fn cb, void* ud) {
    kept_cb = cb;
    kept_ud = ud;
    stale_cb = cb;
    stale_ud = ud;
}

void ke_unregister(ke_fn cb, void* ud) {
    if (kept_cb == cb && kept_ud == ud) {
        kept_cb = 0;
        kept_ud = 0;
    }
}

void ke_poll(void) {
    if (kept_cb) kept_cb(kept_ud);
}

struct call { ke_fn cb; void* ud; };

#ifdef _WIN32
static DWORD WINAPI run(LPVOID argument) {
    struct call* call = (struct call*)argument;
    call->cb(call->ud);
    return 0;
}
#else
static void* run(void* argument) {
    struct call* call = (struct call*)argument;
    call->cb(call->ud);
    return 0;
}
#endif

void ke_poll_elsewhere(void) {
    if (!kept_cb) return;
    struct call call = { kept_cb, kept_ud };
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

void ke_fire_stale(void) {
    if (stale_cb) stale_cb(stale_ud);
}

uint32_t ke_last_tag(void) {
    return (uint32_t)(((uintptr_t)stale_ud) >> 48);
}

void ke_each(ke_fn cb, void* ud, int32_t times) {
    for (int32_t i = 0; i < times; i++) cb(ud);
}
