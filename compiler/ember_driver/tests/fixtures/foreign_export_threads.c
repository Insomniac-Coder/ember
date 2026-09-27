#include "ember_rt.h"
#include <stdlib.h>
#include <string.h>

extern int32_t main_score(void);
extern int32_t any_score(void);
extern int32_t default_score(void);
extern int32_t override_score(void);
static const char* mode;

#if defined(_WIN32)
#include <windows.h>
static DWORD WINAPI worker(LPVOID unused) {
#else
#include <pthread.h>
static void* worker(void* unused) {
#endif
    (void)unused;
    if (strcmp(mode, "reinit") == 0) {
        ember_rt_init(NULL);
        if (main_score() != 41) abort();
    } else {
        /* A repeated init must not transfer the module's thread identity. */
        ember_rt_init(NULL);
        if (any_score() != 42 || override_score() != 44) abort();
        if (strcmp(mode, "wrong") == 0) (void)main_score();
        if (strcmp(mode, "default") == 0) (void)default_score();
    }
    return 0;
}

int main(int argc, char** argv) {
    mode = argc > 1 ? argv[1] : "allowed";
    if (strcmp(mode, "uninitialized") == 0) (void)main_score();
    ember_rt_init(NULL);
    if (main_score() != 41 || default_score() != 43) return 1;
    ember_rt_thread_detach();
    /* Detachment does not change which thread initialized the module. */
    if (main_score() != 41) return 2;
    if (strcmp(mode, "reinit") == 0) ember_rt_shutdown();
#if defined(_WIN32)
    HANDLE thread = CreateThread(NULL, 0, worker, NULL, 0, NULL);
    if (thread == NULL) return 3;
    if (WaitForSingleObject(thread, 10000) != WAIT_OBJECT_0) return 4;
    DWORD result;
    if (!GetExitCodeThread(thread, &result) || result != 0) return 5;
    CloseHandle(thread);
#else
    pthread_t thread;
    if (pthread_create(&thread, NULL, worker, NULL) != 0) return 3;
    void* result;
    if (pthread_join(thread, &result) != 0 || result != NULL) return 5;
#endif
    /* Reinitialization on the worker revokes the old main thread. */
    if (strcmp(mode, "reinit") == 0) (void)main_score();
    ember_rt_shutdown();
    return 0;
}
