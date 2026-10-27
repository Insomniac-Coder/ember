/* pingpong.c: how long does one cache line take to go from one performance core to another and back?
   Two threads, each pinned to one core, take turns writing one shared word (a locked exchange each way).
   Prints one line per pair of the laptop's eight performance cores: the round trip in nanoseconds
   (the median of 7 runs of 200,000 round trips). */
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <intrin.h>
#include <stdio.h>
#include <stdlib.h>

static const int pcores[8] = {0, 1, 10, 11, 12, 13, 22, 23};
static __declspec(align(64)) volatile long turn;
static volatile long go;
#define ROUNDS 200000

typedef struct { int core; int me; } Arg;
static DWORD WINAPI player(LPVOID p) {
    Arg *a = (Arg *)p;
    SetThreadAffinityMask(GetCurrentThread(), (DWORD_PTR)1 << a->core);
    _InterlockedIncrement(&go);
    while (go < 2) _mm_pause();
    for (int i = 0; i < ROUNDS; i++) {
        while (turn != a->me) _mm_pause();
        _InterlockedExchange(&turn, 1 - a->me);
    }
    return 0;
}

static int cmp(const void *a, const void *b) { double x = *(const double *)a, y = *(const double *)b; return x < y ? -1 : x > y; }

int main(void) {
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    LARGE_INTEGER f; QueryPerformanceFrequency(&f);
    for (int i = 0; i < 8; i++) for (int j = i + 1; j < 8; j++) {
        double ns[7];
        for (int r = 0; r < 7; r++) {
            turn = 0; go = 0;
            Arg a = {pcores[i], 0}, b = {pcores[j], 1};
            LARGE_INTEGER t0, t1;
            HANDLE h[2];
            QueryPerformanceCounter(&t0);
            h[0] = CreateThread(NULL, 0, player, &a, 0, NULL);
            h[1] = CreateThread(NULL, 0, player, &b, 0, NULL);
            WaitForMultipleObjects(2, h, TRUE, INFINITE);
            QueryPerformanceCounter(&t1);
            CloseHandle(h[0]); CloseHandle(h[1]);
            ns[r] = (double)(t1.QuadPart - t0.QuadPart) * 1e9 / (double)f.QuadPart / ROUNDS;
        }
        qsort(ns, 7, sizeof(double), cmp);
        printf("cpu %2d - cpu %2d: round trip %.1f ns (min %.1f, max %.1f)\n", pcores[i], pcores[j], ns[3], ns[0], ns[6]);
    }
    return 0;
}
