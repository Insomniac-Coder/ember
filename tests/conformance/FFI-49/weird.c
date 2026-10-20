/* `$` is no character of standard C's identifiers; MSVC, gcc and clang take it. */
#include "weird.h"
int weird$twice(int x) { return 2 * x; }
