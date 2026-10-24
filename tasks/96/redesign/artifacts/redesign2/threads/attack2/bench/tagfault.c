/* tagfault.c: what a C library does when it treats the trampoline's user data as a buffer address
 * (the misinferred contract of probes/y17: `async_read(fd, buf, n, done)` with `buf` taken for user data).
 *
 * Case 1, untagged handle (the answers report's encoding, `slot | generation << 32`): the value is an
 * ordinary user-space address (0x0000000900000005 for slot 5, generation 9). Here that address is mapped,
 * as any heap or file mapping may be, so C's write lands in it and nothing notices.
 * Case 2, tagged handle (this attack's fix): the same slot and generation with the tag 0x4080 in the top
 * 16 bits. The value is not a canonical address, so the first write faults (0xC0000005) before any byte is
 * stored; Ember's handler would turn that into an abort naming the call.
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 tagfault.c -o bin/tagfault.exe */
#include "../../../../redesign/bench/harness.h"
#include <string.h>

/* The "C library": fills the caller's buffer, as a read into `buf` would. */
static __declspec(noinline) void c_library_read_into(void *buf, size_t n) { memset(buf, 0xAB, n); }

static DWORD try_write(void *addr, size_t n) {
    __try { c_library_read_into(addr, n); }
    __except (EXCEPTION_EXECUTE_HANDLER) { return GetExceptionCode(); }
    return 0;
}

int main(void) {
    h_start();
    uintptr_t slot = 5, gen = 9;
    uintptr_t untagged = (gen << 32) | slot;
    uintptr_t tagged = ((uintptr_t)(0x40800000u | gen) << 32) | slot;

    /* Some mapping that happens to cover the untagged value (a heap segment or a mapped file could). */
    uint8_t *page = (uint8_t *)VirtualAlloc((void *)(untagged & ~(uintptr_t)0xFFFF), 1 << 16,
                                            MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
    if (!page) { printf("could not map the page at %p; case 1 not shown\n", (void *)untagged); }
    else {
        memset(page, 0, 1 << 16);
        DWORD code = try_write((void *)untagged, 4096);
        size_t changed = 0;
        for (size_t i = 0; i < (1 << 16); i++) changed += page[i] != 0;
        printf("untagged user data %p: exception 0x%08lX, bytes of an unrelated mapping overwritten: %zu\n",
               (void *)untagged, (unsigned long)code, changed);
    }
    DWORD code = try_write((void *)tagged, 4096);
    printf("tagged user data   %p: exception 0x%08lX (0xC0000005 = access violation), nothing written\n",
           (void *)tagged, (unsigned long)code);
    h_report("tagfault", code);
    return 0;
}
