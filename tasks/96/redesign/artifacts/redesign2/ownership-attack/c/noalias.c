/* noalias.c: field effects rule 5 checks only reshapes between arguments, so `f(c, c)` is accepted when
 * f only writes numbers. The report says the backends "may still assume they do not overlap"; this is
 * what that assumption (C's restrict, LLVM's noalias) does to the accepted call.
 * Ember:   fn reset_then_set(a: Counter, b: Counter) -> int:  a.n = 0; b.n = 1; return a.n
 *          reset_then_set(c, c)        # Python: 1
 * Build: clang -O2 noalias.c -o noalias.exe */
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
typedef struct { long long n; } Counter;
__declspec(noinline) long long with_noalias(Counter *restrict a, Counter *restrict b) { a->n = 0; b->n = 1; return a->n; }
__declspec(noinline) long long without_noalias(Counter *a, Counter *b) { a->n = 0; b->n = 1; return a->n; }
int main(void) {
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    Counter c = {5}, d = {5};
    long long r1 = with_noalias(&c, &c);
    long long r2 = without_noalias(&d, &d);
    printf("reset_then_set(c, c): with noalias %lld (c.n is %lld), without %lld (d.n is %lld)\n", r1, c.n, r2, d.n);
    return 0;
}
