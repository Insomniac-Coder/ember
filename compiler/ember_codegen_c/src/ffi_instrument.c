/* `[FFI-37]`, `[FFI-37a]` — the instrumentation `ember test --instrument-ffi` links into a test
 * binary: while a foreign function runs (between `@RT@ffi_enter` and `@RT@ffi_leave`, which the
 * program wraps around every call it makes to one), the allocation, lock, wait and I/O entry points
 * below record that the function reached them. On Windows they are the import-table entries of
 * every module loaded at start (the C runtime's allocator, the process heap, virtual memory, the
 * kernel's locks, waits and file I/O); on Linux the C library's functions of the same kinds,
 * defined here so the executable's own calls and the C library's interposable ones reach them.
 * What runs through anything else (a custom pool, a driver, a module loaded later, a system call
 * made directly) is structurally unobservable: the evidence says so, and counts nothing for it.
 * At the end of a test the counts are written to the file `@PREFIX@_FFI_EVIDENCE` names. */
#if defined(_WIN32)
#define _CRT_SECURE_NO_WARNINGS
#else
#define _GNU_SOURCE
/* The C library's checked wrappers would be inline definitions of the very names defined here. */
#ifdef _FORTIFY_SOURCE
#undef _FORTIFY_SOURCE
#endif
#endif
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <psapi.h>
#else
#include <dlfcn.h>
#include <fcntl.h>
#include <poll.h>
#include <pthread.h>
#include <time.h>
#include <unistd.h>
#endif

#define FFI_ALLOC 1u
#define FFI_LOCK 2u
#define FFI_BLOCK 4u
#define FFI_IO 8u

typedef struct @RT@ffi_function {
    const char* name;
    const void* address;
} @RT@ffi_function;

extern const @RT@ffi_function @RT@ffi_functions[];
extern const uint32_t @RT@ffi_function_count;
extern const uint32_t @RT@ffi_site_count;

static uint32_t* g_calls;
static uint32_t* g_observed;
static uint32_t* g_site_hits;
static int g_ready;

#if defined(_MSC_VER)
static __declspec(thread) uint32_t g_current;
#else
static _Thread_local uint32_t g_current;
#endif

#if defined(_MSC_VER) && !defined(__clang__)
long _InterlockedOr(long volatile* word, long bits);
long _InterlockedIncrement(long volatile* word);
#pragma intrinsic(_InterlockedOr, _InterlockedIncrement)
#define FFI_OR(word, bits) ((void)_InterlockedOr((long volatile*)(word), (long)(bits)))
#define FFI_COUNT(word) ((void)_InterlockedIncrement((long volatile*)(word)))
#else
#define FFI_OR(word, bits) ((void)__atomic_fetch_or((word), (bits), __ATOMIC_RELAXED))
#define FFI_COUNT(word) ((void)__atomic_fetch_add((word), 1u, __ATOMIC_RELAXED))
#endif

/* What the function running now on this thread reached. */
static void note(uint32_t kind) {
    uint32_t current = g_current;
    if (current != 0 && g_ready) {
        FFI_OR(&g_observed[current - 1], kind);
    }
}

uint32_t @RT@ffi_enter(uint32_t function, uint32_t site) {
    uint32_t previous = g_current;
    if (g_ready) {
        FFI_COUNT(&g_calls[function]);
        FFI_COUNT(&g_site_hits[site]);
    }
    g_current = function + 1;
    return previous;
}

void @RT@ffi_leave(uint32_t previous) {
    g_current = previous;
}

#if defined(_WIN32)
/* -- Windows: the import tables of every module loaded at start ----------------------------- */

static void* (__cdecl* real_malloc)(size_t);
static void* (__cdecl* real_calloc)(size_t, size_t);
static void* (__cdecl* real_realloc)(void*, size_t);
static void* (__cdecl* real_aligned_malloc)(size_t, size_t);
static LPVOID (WINAPI* real_HeapAlloc)(HANDLE, DWORD, SIZE_T);
static LPVOID (WINAPI* real_HeapReAlloc)(HANDLE, DWORD, LPVOID, SIZE_T);
static LPVOID (WINAPI* real_VirtualAlloc)(LPVOID, SIZE_T, DWORD, DWORD);
static void (WINAPI* real_EnterCriticalSection)(LPCRITICAL_SECTION);
static void (WINAPI* real_AcquireSRWLockExclusive)(PSRWLOCK);
static void (WINAPI* real_AcquireSRWLockShared)(PSRWLOCK);
static void (WINAPI* real_Sleep)(DWORD);
static DWORD (WINAPI* real_SleepEx)(DWORD, BOOL);
static DWORD (WINAPI* real_WaitForSingleObject)(HANDLE, DWORD);
static DWORD (WINAPI* real_WaitForSingleObjectEx)(HANDLE, DWORD, BOOL);
static DWORD (WINAPI* real_WaitForMultipleObjects)(DWORD, const HANDLE*, BOOL, DWORD);
static BOOL (WINAPI* real_SleepConditionVariableSRW)(PCONDITION_VARIABLE, PSRWLOCK, DWORD, ULONG);
static BOOL (WINAPI* real_SleepConditionVariableCS)(PCONDITION_VARIABLE, PCRITICAL_SECTION, DWORD);
static BOOL (WINAPI* real_ReadFile)(HANDLE, LPVOID, DWORD, LPDWORD, LPOVERLAPPED);
static BOOL (WINAPI* real_WriteFile)(HANDLE, LPCVOID, DWORD, LPDWORD, LPOVERLAPPED);
static HANDLE (WINAPI* real_CreateFileW)(LPCWSTR, DWORD, DWORD, LPSECURITY_ATTRIBUTES, DWORD, DWORD, HANDLE);
static HANDLE (WINAPI* real_CreateFileA)(LPCSTR, DWORD, DWORD, LPSECURITY_ATTRIBUTES, DWORD, DWORD, HANDLE);
static BOOL (WINAPI* real_WriteConsoleW)(HANDLE, const VOID*, DWORD, LPDWORD, LPVOID);
static BOOL (WINAPI* real_WriteConsoleA)(HANDLE, const VOID*, DWORD, LPDWORD, LPVOID);

static void* __cdecl hook_malloc(size_t n) { note(FFI_ALLOC); return real_malloc(n); }
static void* __cdecl hook_calloc(size_t n, size_t size) { note(FFI_ALLOC); return real_calloc(n, size); }
static void* __cdecl hook_realloc(void* p, size_t n) { note(FFI_ALLOC); return real_realloc(p, n); }
static void* __cdecl hook_aligned_malloc(size_t n, size_t align) { note(FFI_ALLOC); return real_aligned_malloc(n, align); }
static LPVOID WINAPI hook_HeapAlloc(HANDLE h, DWORD f, SIZE_T n) { note(FFI_ALLOC); return real_HeapAlloc(h, f, n); }
static LPVOID WINAPI hook_HeapReAlloc(HANDLE h, DWORD f, LPVOID p, SIZE_T n) { note(FFI_ALLOC); return real_HeapReAlloc(h, f, p, n); }
static LPVOID WINAPI hook_VirtualAlloc(LPVOID p, SIZE_T n, DWORD t, DWORD protect) { note(FFI_ALLOC); return real_VirtualAlloc(p, n, t, protect); }
static void WINAPI hook_EnterCriticalSection(LPCRITICAL_SECTION c) { note(FFI_LOCK); real_EnterCriticalSection(c); }
static void WINAPI hook_AcquireSRWLockExclusive(PSRWLOCK l) { note(FFI_LOCK); real_AcquireSRWLockExclusive(l); }
static void WINAPI hook_AcquireSRWLockShared(PSRWLOCK l) { note(FFI_LOCK); real_AcquireSRWLockShared(l); }
static void WINAPI hook_Sleep(DWORD ms) { note(FFI_BLOCK); real_Sleep(ms); }
static DWORD WINAPI hook_SleepEx(DWORD ms, BOOL alert) { note(FFI_BLOCK); return real_SleepEx(ms, alert); }
static DWORD WINAPI hook_WaitForSingleObject(HANDLE h, DWORD ms) { note(FFI_BLOCK); return real_WaitForSingleObject(h, ms); }
static DWORD WINAPI hook_WaitForSingleObjectEx(HANDLE h, DWORD ms, BOOL alert) { note(FFI_BLOCK); return real_WaitForSingleObjectEx(h, ms, alert); }
static DWORD WINAPI hook_WaitForMultipleObjects(DWORD n, const HANDLE* h, BOOL all, DWORD ms) { note(FFI_BLOCK); return real_WaitForMultipleObjects(n, h, all, ms); }
static BOOL WINAPI hook_SleepConditionVariableSRW(PCONDITION_VARIABLE c, PSRWLOCK l, DWORD ms, ULONG f) { note(FFI_BLOCK); return real_SleepConditionVariableSRW(c, l, ms, f); }
static BOOL WINAPI hook_SleepConditionVariableCS(PCONDITION_VARIABLE c, PCRITICAL_SECTION s, DWORD ms) { note(FFI_BLOCK); return real_SleepConditionVariableCS(c, s, ms); }
static BOOL WINAPI hook_ReadFile(HANDLE h, LPVOID b, DWORD n, LPDWORD r, LPOVERLAPPED o) { note(FFI_IO); return real_ReadFile(h, b, n, r, o); }
static BOOL WINAPI hook_WriteFile(HANDLE h, LPCVOID b, DWORD n, LPDWORD w, LPOVERLAPPED o) { note(FFI_IO); return real_WriteFile(h, b, n, w, o); }
static HANDLE WINAPI hook_CreateFileW(LPCWSTR p, DWORD a, DWORD s, LPSECURITY_ATTRIBUTES sa, DWORD c, DWORD f, HANDLE t) { note(FFI_IO); return real_CreateFileW(p, a, s, sa, c, f, t); }
static HANDLE WINAPI hook_CreateFileA(LPCSTR p, DWORD a, DWORD s, LPSECURITY_ATTRIBUTES sa, DWORD c, DWORD f, HANDLE t) { note(FFI_IO); return real_CreateFileA(p, a, s, sa, c, f, t); }
static BOOL WINAPI hook_WriteConsoleW(HANDLE h, const VOID* b, DWORD n, LPDWORD w, LPVOID r) { note(FFI_IO); return real_WriteConsoleW(h, b, n, w, r); }
static BOOL WINAPI hook_WriteConsoleA(HANDLE h, const VOID* b, DWORD n, LPDWORD w, LPVOID r) { note(FFI_IO); return real_WriteConsoleA(h, b, n, w, r); }

typedef struct hook {
    const char* module;
    const char* name;
    void** real;
    void* replacement;
} hook;

#define HOOK(module, name) { module, #name, (void**)&real_##name, (void*)hook_##name }
static const hook g_hooks[] = {
    HOOK("ucrtbase.dll", malloc), HOOK("ucrtbase.dll", calloc), HOOK("ucrtbase.dll", realloc),
    { "ucrtbase.dll", "_aligned_malloc", (void**)&real_aligned_malloc, (void*)hook_aligned_malloc },
    HOOK("kernel32.dll", HeapAlloc), HOOK("kernel32.dll", HeapReAlloc), HOOK("kernel32.dll", VirtualAlloc),
    HOOK("kernel32.dll", EnterCriticalSection), HOOK("kernel32.dll", AcquireSRWLockExclusive),
    HOOK("kernel32.dll", AcquireSRWLockShared),
    HOOK("kernel32.dll", Sleep), HOOK("kernel32.dll", SleepEx), HOOK("kernel32.dll", WaitForSingleObject),
    HOOK("kernel32.dll", WaitForSingleObjectEx), HOOK("kernel32.dll", WaitForMultipleObjects),
    HOOK("kernel32.dll", SleepConditionVariableSRW), HOOK("kernel32.dll", SleepConditionVariableCS),
    HOOK("kernel32.dll", ReadFile), HOOK("kernel32.dll", WriteFile), HOOK("kernel32.dll", CreateFileW),
    HOOK("kernel32.dll", CreateFileA), HOOK("kernel32.dll", WriteConsoleW), HOOK("kernel32.dll", WriteConsoleA),
};
#define HOOK_COUNT (sizeof g_hooks / sizeof g_hooks[0])

static const char g_intercepted[] =
    "alloc: malloc, calloc, realloc, _aligned_malloc (ucrtbase), HeapAlloc, HeapReAlloc, VirtualAlloc; "
    "lock: EnterCriticalSection, AcquireSRWLockExclusive, AcquireSRWLockShared; "
    "block: Sleep, SleepEx, WaitForSingleObject(Ex), WaitForMultipleObjects, SleepConditionVariableSRW/CS; "
    "io: ReadFile, WriteFile, CreateFileW/A, WriteConsoleW/A; "
    "each through the import tables of the modules loaded when the test starts";

/* Point each module's import-table entry for a hooked function at its hook. */
static void patch_module(HMODULE module) {
    BYTE* base = (BYTE*)module;
    if (((IMAGE_DOS_HEADER*)base)->e_magic != IMAGE_DOS_SIGNATURE) {
        return;
    }
    IMAGE_NT_HEADERS* nt = (IMAGE_NT_HEADERS*)(base + ((IMAGE_DOS_HEADER*)base)->e_lfanew);
    IMAGE_DATA_DIRECTORY imports = nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
    if (imports.VirtualAddress == 0) {
        return;
    }
    for (IMAGE_IMPORT_DESCRIPTOR* entry = (IMAGE_IMPORT_DESCRIPTOR*)(base + imports.VirtualAddress); entry->Name != 0; ++entry) {
        for (IMAGE_THUNK_DATA* thunk = (IMAGE_THUNK_DATA*)(base + entry->FirstThunk); thunk->u1.Function != 0; ++thunk) {
            void** slot = (void**)&thunk->u1.Function;
            for (size_t i = 0; i < HOOK_COUNT; ++i) {
                if (*g_hooks[i].real != NULL && *slot == *g_hooks[i].real) {
                    DWORD old;
                    if (VirtualProtect(slot, sizeof *slot, PAGE_READWRITE, &old)) {
                        *slot = g_hooks[i].replacement;
                        VirtualProtect(slot, sizeof *slot, old, &old);
                    }
                }
            }
        }
    }
}

static void install(void) {
    for (size_t i = 0; i < HOOK_COUNT; ++i) {
        HMODULE module = GetModuleHandleA(g_hooks[i].module);
        *g_hooks[i].real = module != NULL ? (void*)GetProcAddress(module, g_hooks[i].name) : NULL;
    }
    HMODULE modules[1024];
    DWORD needed = 0;
    if (!K32EnumProcessModules(GetCurrentProcess(), modules, sizeof modules, &needed)) {
        return;
    }
    for (DWORD i = 0; i < needed / sizeof modules[0] && i < 1024; ++i) {
        patch_module(modules[i]);
    }
}

static void module_of(const void* address, char* path, size_t size) {
    HMODULE module = NULL;
    path[0] = '\0';
    /* A function another module exports is reached through a jump in this one
     * (`jmp [rip + n]` through the import table): the library is the jump's target's. */
    const unsigned char* code = (const unsigned char*)address;
    if (code != NULL && code[0] == 0xFF && code[1] == 0x25) {
        int32_t offset;
        memcpy(&offset, code + 2, sizeof offset);
        address = *(void* const*)(code + 6 + offset);
    }
    if (address != NULL && GetModuleHandleExA(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            (LPCSTR)address, &module)) {
        GetModuleFileNameA(module, path, (DWORD)size);
    }
}

#else
/* -- Linux (glibc): the C library's entry points, defined in the executable ------------------- */

extern void* __libc_malloc(size_t);
extern void* __libc_calloc(size_t, size_t);
extern void* __libc_realloc(void*, size_t);
extern void* __libc_memalign(size_t, size_t);

void* malloc(size_t n) { note(FFI_ALLOC); return __libc_malloc(n); }
void* calloc(size_t n, size_t size) { note(FFI_ALLOC); return __libc_calloc(n, size); }
void* realloc(void* p, size_t n) { note(FFI_ALLOC); return __libc_realloc(p, n); }
void* aligned_alloc(size_t align, size_t n) { note(FFI_ALLOC); return __libc_memalign(align, n); }
int posix_memalign(void** out, size_t align, size_t n) {
    note(FFI_ALLOC);
    void* p = __libc_memalign(align, n);
    if (p == NULL) return 12;
    *out = p;
    return 0;
}

#define REAL(name) static __typeof__(name)* real_##name; \
    if (real_##name == NULL) real_##name = (__typeof__(name)*)dlsym(RTLD_NEXT, #name)

int pthread_mutex_lock(pthread_mutex_t* m) { note(FFI_LOCK); REAL(pthread_mutex_lock); return real_pthread_mutex_lock(m); }
int pthread_rwlock_rdlock(pthread_rwlock_t* l) { note(FFI_LOCK); REAL(pthread_rwlock_rdlock); return real_pthread_rwlock_rdlock(l); }
int pthread_rwlock_wrlock(pthread_rwlock_t* l) { note(FFI_LOCK); REAL(pthread_rwlock_wrlock); return real_pthread_rwlock_wrlock(l); }
int pthread_cond_wait(pthread_cond_t* c, pthread_mutex_t* m) { note(FFI_BLOCK); REAL(pthread_cond_wait); return real_pthread_cond_wait(c, m); }
int pthread_cond_timedwait(pthread_cond_t* c, pthread_mutex_t* m, const struct timespec* t) { note(FFI_BLOCK); REAL(pthread_cond_timedwait); return real_pthread_cond_timedwait(c, m, t); }
int pthread_join(pthread_t t, void** r) { note(FFI_BLOCK); REAL(pthread_join); return real_pthread_join(t, r); }
int nanosleep(const struct timespec* t, struct timespec* r) { note(FFI_BLOCK); REAL(nanosleep); return real_nanosleep(t, r); }
int usleep(useconds_t us) { note(FFI_BLOCK); REAL(usleep); return real_usleep(us); }
unsigned int sleep(unsigned int s) { note(FFI_BLOCK); REAL(sleep); return real_sleep(s); }
int poll(struct pollfd* f, nfds_t n, int ms) { note(FFI_BLOCK); REAL(poll); return real_poll(f, n, ms); }
ssize_t read(int fd, void* b, size_t n) { note(FFI_IO); REAL(read); return real_read(fd, b, n); }
ssize_t write(int fd, const void* b, size_t n) { note(FFI_IO); REAL(write); return real_write(fd, b, n); }
FILE* fopen(const char* p, const char* m) { note(FFI_IO); REAL(fopen); return real_fopen(p, m); }
size_t fread(void* b, size_t s, size_t n, FILE* f) { note(FFI_IO); REAL(fread); return real_fread(b, s, n, f); }
size_t fwrite(const void* b, size_t s, size_t n, FILE* f) { note(FFI_IO); REAL(fwrite); return real_fwrite(b, s, n, f); }
int fputs(const char* s, FILE* f) { note(FFI_IO); REAL(fputs); return real_fputs(s, f); }
int puts(const char* s) { note(FFI_IO); REAL(puts); return real_puts(s); }
int fflush(FILE* f) { note(FFI_IO); REAL(fflush); return real_fflush(f); }

static const char g_intercepted[] =
    "alloc: malloc, calloc, realloc, aligned_alloc, posix_memalign; "
    "lock: pthread_mutex_lock, pthread_rwlock_rdlock, pthread_rwlock_wrlock; "
    "block: pthread_cond_wait, pthread_cond_timedwait, pthread_join, nanosleep, usleep, sleep, poll; "
    "io: read, write, fopen, fread, fwrite, fputs, puts, fflush; "
    "each where the executable calls it, or the C library calls it interposably";

static void install(void) {}

static void module_of(const void* address, char* path, size_t size) {
    Dl_info info;
    path[0] = '\0';
    if (address != NULL && dladdr(address, &info) && info.dli_fname != NULL) {
        snprintf(path, size, "%s", info.dli_fname);
    }
}
#endif

/* Before the test: the counters, and on Windows the import tables patched. */
void @RT@ffi_instrument_begin(void) {
    g_calls = (uint32_t*)calloc(@RT@ffi_function_count + 1, sizeof *g_calls);
    g_observed = (uint32_t*)calloc(@RT@ffi_function_count + 1, sizeof *g_observed);
    g_site_hits = (uint32_t*)calloc(@RT@ffi_site_count + 1, sizeof *g_site_hits);
    if (g_calls == NULL || g_observed == NULL || g_site_hits == NULL) {
        return;
    }
    install();
    g_ready = 1;
}

/* After the test: what was observed, for the runner. */
void @RT@ffi_instrument_end(void) {
    if (!g_ready) {
        return;
    }
    g_ready = 0;
    const char* path = getenv("@PREFIX@_FFI_EVIDENCE");
    if (path == NULL) {
        return;
    }
    FILE* out = fopen(path, "w");
    if (out == NULL) {
        return;
    }
    fprintf(out, "intercepted %s\n", g_intercepted);
    for (uint32_t i = 0; i < @RT@ffi_function_count; ++i) {
        char module[1024];
        module_of(@RT@ffi_functions[i].address, module, sizeof module);
        fprintf(out, "function %u %u %u %s\n", (unsigned)i, (unsigned)g_calls[i], (unsigned)g_observed[i], module);
    }
    for (uint32_t i = 0; i < @RT@ffi_site_count; ++i) {
        fprintf(out, "site %u %u\n", (unsigned)i, (unsigned)g_site_hits[i]);
    }
    fclose(out);
}
