/* salvage_samevar_generic.c: a bound on verdict 88's code growth.

   Idea: a function with k `ref mut` parameters of one type may need one copy per way of grouping its
   names, the Bell number B(k) (2, 5, 15, 52 for k = 2 .. 5; samevar_size.c: all 15 copies of `mix4`
   are 8,324 bytes of code against 719 for the normal one, clang -O2). One copy compiled with no
   noalias at all is correct for every grouping, since nothing in it assumes two names differ. So
   the compiler can make at most two
   copies of any function: the normal one (every `ref mut` noalias) for calls with different places,
   and one alias-safe copy for every call that names one whole variable more than once. Those calls
   then run at plain C's speed for the same call (plain_samevar `one`), not the one-name copy's.
   Used when a function would need more than one copy with merged names (a compiler limit).

   Safety: a copy without noalias makes no promise the call could break; every check stays. */
#define GENERIC_COPY 1
#define NAME "salvage_samevar_generic"
#include "model_samevar.c"
