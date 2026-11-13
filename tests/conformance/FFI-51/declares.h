/* A header that only declares: a type, a macro and a `static inline` function. */
#ifndef DECLARES_H
#define DECLARES_H
#define DECLARES_SIX 6
typedef struct Pair { int a; int b; } Pair;
static inline int declares_twice(int x) { return 2 * x; }
#endif
