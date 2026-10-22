/* The C side of the FFI-8 tests of `long double`: values Ember keeps and hands back, in a struct
   too, without arithmetic of its own. */
#ifndef PRECISE_H
#define PRECISE_H
struct Measure { int tag; long double value; };
long double precise_third(void);
long double precise_twice(long double x);
double precise_to_double(long double x);
struct Measure precise_measure(int tag);
#endif
