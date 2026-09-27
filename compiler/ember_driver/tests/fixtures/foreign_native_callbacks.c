/* Including the emitted translation unit keeps the host's declarations
 * exactly matched, while the separate tests for C record exports cover
 * independently compiled declarations. */
#include "program.c"
#include "../../../../runtime/ember_rt/src/ember_rt.c"

int32_t ember_native_fn_value_0 = 73;
int32_t ember_native_fn_c_entry_1 = 77;

static int exercise_callbacks(void) {
    /* Factories attach, but their returned callbacks must attach themselves. */
    int32_t (*named)(int32_t) = make_named();
    int32_t (*inline_fn)(int32_t) = make_inline();
    int32_t (*selected_one)(int32_t) = make_selected(0);
    int32_t (*selected_two)(int32_t) = make_selected(1);
    int32_t (*pair_fn)(em_Pair) = make_pair();
    int64_t (*large_fn)(em_Large) = make_large();
    int32_t (*mut_fn)(em_Pair*) = make_mut();
    em_Pair (*return_fn)(int32_t) = make_returned();
    int32_t (*nested_fn)(int32_t (*)(int32_t), int32_t) = make_nested();
    void (*void_fn)(int32_t) = make_void();
#define CHECK_CALLBACK(expression, expected, code) do { \
    ember_rt_thread_detach(); \
    if ((expression) != (expected)) return (code); \
    if (!runtime_thread_attached()) return (code) + 100; \
} while (0)
    CHECK_CALLBACK(named(41), 42, 1);
    CHECK_CALLBACK(inline_fn(40), 43, 2);
    CHECK_CALLBACK(selected_one(41), 42, 3);
    CHECK_CALLBACK(selected_two(41), 43, 4);
    em_Pair pair = { 19, 23 };
    CHECK_CALLBACK(pair_fn(pair), 42, 5);
    if (pair.left != 19 || pair.right != 23) return 6;
    em_Large large = { 10, 20, 12 };
    CHECK_CALLBACK(large_fn(large), 42, 7);
    CHECK_CALLBACK(mut_fn(&pair), 52, 8);
    if (pair.left != 29) return 9;
    ember_rt_thread_detach();
    em_Pair returned = return_fn(20);
    if (!runtime_thread_attached()) return 110;
    if (returned.left != 20 || returned.right != 21) return 10;
    CHECK_CALLBACK(nested_fn(named, 41), 42, 11);
    ember_rt_thread_detach();
    void_fn(42);
    if (!runtime_thread_attached()) return 12;
    CHECK_CALLBACK(check_native_identity(), 1, 13);
    CHECK_CALLBACK(reserved_static_value(), 150, 14);
#undef CHECK_CALLBACK
    return 0;
}

#if defined(_WIN32)
#include <windows.h>
static DWORD WINAPI worker(LPVOID unused) {
    (void)unused;
    return (DWORD)exercise_callbacks();
}
#else
#include <pthread.h>
static void* worker(void* unused) {
    (void)unused;
    return (void*)(uintptr_t)exercise_callbacks();
}
#endif

int main(void) {
    ember_rt_init(NULL);
#if defined(_WIN32)
    HANDLE thread = CreateThread(NULL, 0, worker, NULL, 0, NULL);
    if (thread == NULL) return 20;
    if (WaitForSingleObject(thread, 10000) != WAIT_OBJECT_0) return 21;
    DWORD result;
    if (!GetExitCodeThread(thread, &result)) return 22;
    CloseHandle(thread);
    if (result != 0) return (int)result;
#else
    pthread_t thread;
    if (pthread_create(&thread, NULL, worker, NULL) != 0) return 20;
    void* result;
    if (pthread_join(thread, &result) != 0) return 21;
    if (result != NULL) return (int)(uintptr_t)result;
#endif
    ember_rt_shutdown();
    return 0;
}
