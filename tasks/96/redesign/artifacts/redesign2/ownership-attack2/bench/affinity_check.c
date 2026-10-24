/* affinity_check.c: prints the affinity mask this process was created with, to show that a program
 * started by run.ps1 is pinned from its first instruction. Build: clang -O2 affinity_check.c */
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
int main(void) {
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    DWORD_PTR proc = 0, sys = 0;
    GetProcessAffinityMask(GetCurrentProcess(), &proc, &sys);
    printf("affinity-check ms=0 peak_kb=0 checksum=%llu\n", (unsigned long long)proc);
    printf("process mask 0x%llX, system mask 0x%llX\n", (unsigned long long)proc, (unsigned long long)sys);
    return 0;
}
