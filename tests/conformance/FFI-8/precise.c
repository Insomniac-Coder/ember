#include "precise.h"
long double precise_third(void) { return 1.0L / 3.0L; }
long double precise_twice(long double x) { return x * 2.0L; }
double precise_to_double(long double x) { return (double)x; }
struct Measure precise_measure(int tag) { struct Measure m = { tag, (long double)tag / 4.0L }; return m; }
