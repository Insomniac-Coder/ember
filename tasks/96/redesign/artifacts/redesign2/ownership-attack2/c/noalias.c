/* noalias.c: field effects rule 5 checks only reshapes between arguments, so `f(c, c)` is accepted when
 * f only writes numbers. The report says the backends "may still assume they do not overlap"; this is
 * what that assumption (C's restrict, LLVM's noalias) does to the accepted call.
 * Ember:   fn reset_then_set(a: Counter, b: Counter) -> int:  a.n = 0; b.n = 1; return a.n
 *          reset_then_set(c, c)        # Python: 1
 * Second attack: the save. The containers group's A3 rule 2 (phase3b-containers-attack.md, section 9):
 * compile the function twice, once assuming no overlap and once not, and compare at a call that may pass
 * one place twice. `dispatch` is that call.
 * Build: clang -O2 noalias.c -o noalias.exe */
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
typedef struct { long long n; } Counter;
__declspec(noinline) long long with_noalias(Counter *restrict a, Counter *restrict b) { a->n = 0; b->n = 1; return a->n; }
__declspec(noinline) long long without_noalias(Counter *a, Counter *b) { a->n = 0; b->n = 1; return a->n; }
static long long dispatch(Counter *a, Counter *b) {             /* the call site, as emitted */
    return a == b ? without_noalias(a, b) : with_noalias(a, b);
}
int main(void) {
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    Counter c = {5}, d = {5};
    long long r1 = with_noalias(&c, &c);
    long long r2 = without_noalias(&d, &d);
    printf("reset_then_set(c, c): with noalias %lld (c.n is %lld), without %lld (d.n is %lld)\n", r1, c.n, r2, d.n);
    Counter e = {5}, f = {5}, g = {5};
    long long r3 = dispatch(&e, &e), r4 = dispatch(&f, &g);
    printf("the save, two copies and a compare: reset_then_set(e, e) = %lld; reset_then_set(f, g) = %lld (f.n %lld, g.n %lld)\n", r3, r4, f.n, g.n);
    return 0;
}
