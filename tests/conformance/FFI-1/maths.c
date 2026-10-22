#include "maths.h"

int square(int x) { return x * x; }

static const int offsets[4] = { 2, 3, 5, 7 };
int offset(int index) { return offsets[index & 3]; }

static int count;
int counter(void) { return ++count; }
